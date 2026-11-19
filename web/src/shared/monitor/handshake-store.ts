import type { Telemetry } from "../protocol";

/** Pairs tracked at once (mirrors the device's own bound, `handshake::CAPACITY`). */
const CAPACITY = 32;

export interface HandshakeMessage {
  message: number;
  replayCounter: number;
  truncated: boolean;
  eapol: Uint8Array;
  at: number;
}

export interface PmkidRecord {
  /** BSSID of the Access Point. */
  bssid: string;
  /** MAC of the Station (client). */
  client: string;
  /** 32 lowercase hex chars. */
  pmkid: string;
  at: number;
}

export interface HandshakePair {
  bssid: string;
  client: string;
  /** Keyed by message number (1-4); the latest frame of each. */
  messages: Map<number, HandshakeMessage>;
  complete: boolean;
  firstSeen: number;
  lastSeen: number;
}

const key = (bssid: string, client: string) => `${bssid}|${client}`;

/**
 * What the passive handshake listener has reported: on/off state and the captured pairs. A pair holds
 * the raw EAPOL bytes of whichever messages were seen, for the hex export; nothing here is sent anywhere.
 * Also records passively extracted PMKIDs for the Hashcat export.
 */
export class HandshakeState {
  enabled = false;
  private pairs = new Map<string, HandshakePair>();
  private order: string[] = []; // insertion order, for bounding
  /** PMKIDs extracted from EAPOL-Key Message 1 frames, keyed by bssid|pmkid for dedup. */
  private pmkidMap = new Map<string, PmkidRecord>();
  private subs: Array<() => void> = [];

  constructor(private readonly now: () => number = Date.now) {}

  onChange(cb: () => void): void {
    this.subs.push(cb);
  }

  list(): HandshakePair[] {
    return [...this.pairs.values()].sort((a, b) => b.lastSeen - a.lastSeen);
  }

  /** All captured PMKIDs for a given BSSID, newest first. */
  pmkidsFor(bssid: string): PmkidRecord[] {
    return [...this.pmkidMap.values()]
      .filter((r) => r.bssid === bssid)
      .sort((a, b) => b.at - a.at);
  }

  /** All captured PMKIDs across all monitored access points, newest first. */
  allPmkids(): PmkidRecord[] {
    return [...this.pmkidMap.values()].sort((a, b) => b.at - a.at);
  }

  ingest(m: Telemetry): void {
    if (m.type === "handshakeCaptureState") {
      this.enabled = m.enabled;
      if (!m.enabled) {
        this.pairs.clear();
        this.order = [];
        this.pmkidMap.clear();
      }
    } else if (m.type === "handshakeFrame") {
      const k = key(m.bssid, m.client);
      let pair = this.pairs.get(k);
      if (!pair) {
        if (this.order.length >= CAPACITY) {
          const oldest = this.order.shift();
          if (oldest) this.pairs.delete(oldest);
        }
        pair = { bssid: m.bssid, client: m.client, messages: new Map(), complete: false, firstSeen: this.now(), lastSeen: this.now() };
        this.pairs.set(k, pair);
        this.order.push(k);
      }
      pair.messages.set(m.message, { message: m.message, replayCounter: m.replayCounter, truncated: m.truncated, eapol: m.eapol, at: this.now() });
      pair.lastSeen = this.now();
    } else if (m.type === "handshakeComplete") {
      const pair = this.pairs.get(key(m.bssid, m.client));
      if (pair) pair.complete = true;
    } else if (m.type === "pmkidFound") {
      // Dedup by bssid|pmkid so repeated EAPOL M1 retransmissions don't pile up.
      const dk = `${m.bssid}|${m.pmkid}`;
      if (!this.pmkidMap.has(dk)) {
        this.pmkidMap.set(dk, { bssid: m.bssid, client: m.client, pmkid: m.pmkid, at: this.now() });
      }
    } else {
      return;
    }
    this.subs.forEach((f) => f());
  }
}

const toHex = (b: Uint8Array) => [...b].map((x) => x.toString(16).padStart(2, "0")).join(" ");

/** A readable hex dump of one pair's captured messages, in message order, each flagged if truncated. */
export function pairToHex(p: HandshakePair): string {
  const lines = [`# ${p.bssid} <-> ${p.client}${p.complete ? " (complete)" : ""}`];
  for (const m of [...p.messages.values()].sort((a, b) => a.message - b.message)) {
    lines.push(`## message ${m.message}${m.truncated ? " (truncated: longer than the capture cap)" : ""}`);
    lines.push(toHex(m.eapol));
  }
  return lines.join("\n") + "\n";
}

/**
 * Format a PMKID record as Hashcat mode 22000 (`WPA*01*`) line.
 *
 * Format: `WPA*01*<pmkid>*<bssid_no_sep>*<client_no_sep>*<essid_hex>***`
 * See https://hashcat.net/wiki/doku.php?id=hashcat (mode 22000).
 *
 * @param r      - PMKID record (pmkid is 32 hex chars; bssid/client are colon-separated).
 * @param essid  - SSID of the network (UTF-8 string), encoded to hex in the output.
 */
export function pmkidToHashcat22000(r: PmkidRecord, essid: string): string {
  const stripColons = (mac: string) => mac.replace(/:/g, "");
  const essidHex = [...new TextEncoder().encode(essid)]
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
  return `WPA*01*${r.pmkid}*${stripColons(r.bssid)}*${stripColons(r.client)}*${essidHex}***`;
}

/**
 * Format a 4-way handshake as a Hashcat mode 22000 (`WPA*02*`) line.
 *
 * Format: `WPA*02*<MIC>*<MAC_AP>*<MAC_CLIENT>*<ESSID_HEX>*<ANONCE>*<EAPOL_ZEROED_MIC>*<MESSAGE_PAIR>`
 * See https://hashcat.net/wiki/doku.php?id=cracking_wpawpa2 (mode 22000).
 *
 * @param p      - Handshake pair containing messages (must include M1 or M3 for ANonce, and M2 for MIC & client EAPOL)
 * @param essid  - SSID of the network (UTF-8 string), encoded to lowercase hex in the output
 * @returns Hashcat 22000 line, or null if required messages or fields are missing/short
 */
export function handshakeToHashcat22000(p: HandshakePair, essid: string): string | null {
  // ANonce comes from Message 1 (or Message 3 if M1 wasn't captured).
  const m1 = p.messages.get(1) ?? p.messages.get(3);
  const m2 = p.messages.get(2);
  if (!m1 || !m2) return null;

  // In EAPOL-Key (IEEE 802.11i):
  // Offset 17..49 (32 bytes) = Key Nonce (ANonce in M1/M3).
  // Offset 81..97 (16 bytes) = Key MIC in M2.
  // M2 must be at least 97 bytes to contain the MIC.
  if (m1.eapol.length < 49 || m2.eapol.length < 97) return null;

  const hexNoSep = (b: Uint8Array) =>
    [...b].map((x) => x.toString(16).padStart(2, "0")).join("");

  const stripColons = (mac: string) => mac.replace(/[^0-9a-f]/gi, "").toLowerCase();

  const anonce = hexNoSep(m1.eapol.slice(17, 49));
  const mic = hexNoSep(m2.eapol.slice(81, 97));

  // Clone M2's EAPOL and nullify the 16 bytes of MIC (offset 81..97)
  const eapolZeroed = new Uint8Array(m2.eapol);
  eapolZeroed.fill(0, 81, 97);
  const eapolHex = hexNoSep(eapolZeroed);

  const apMac = stripColons(p.bssid);
  const staMac = stripColons(p.client);
  const essidHex = hexNoSep(new TextEncoder().encode(essid));

  // Message pair indicator: "00" for M1+M2, "02" for M3+M2
  const messagePair = p.messages.has(1) ? "00" : "02";

  return `WPA*02*${mic}*${apMac}*${staMac}*${essidHex}*${anonce}*${eapolHex}*${messagePair}`;
}

/**
 * Consolidates all crackable 4-way handshakes and clientless PMKIDs into a single
 * Hashcat Mode 22000 multi-line text format.
 *
 * @param pairs - Handshake pairs
 * @param pmkids - PMKID records
 * @param getEssid - Function resolving BSSID to SSID name (defaults to empty string if unknown)
 * @returns Multi-line text string with one hash per line, or empty string if none crackable
 */
export function exportAll22000(
  pairs: readonly HandshakePair[],
  pmkids: readonly PmkidRecord[],
  getEssid: (bssid: string) => string = () => "",
): string {
  const lines: string[] = [];

  for (const pair of pairs) {
    const essid = getEssid(pair.bssid);
    const line = handshakeToHashcat22000(pair, essid);
    if (line) lines.push(line);
  }

  for (const record of pmkids) {
    const essid = getEssid(record.bssid);
    lines.push(pmkidToHashcat22000(record, essid));
  }

  return lines.length > 0 ? lines.join("\n") + "\n" : "";
}
