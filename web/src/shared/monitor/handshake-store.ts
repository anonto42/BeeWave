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
