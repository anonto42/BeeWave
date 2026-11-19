import type { HandshakeMessage, HandshakePair } from "./handshake-store";

/** `DLT_IEEE802_11`: plain 802.11 frames, no radiotap header (we were never given radio metadata to put in one). */
const LINKTYPE_IEEE802_11 = 105;
const PCAP_MAGIC_LE = 0xa1b2c3d4;
/// The constant 8-byte LLC/SNAP header in front of every EAPOL frame (mirrors the device parser's own constant).
const LLC_SNAP_EAPOL = Uint8Array.from([0xaa, 0xaa, 0x03, 0x00, 0x00, 0x00, 0x88, 0x8e]);

function macBytes(mac: string): Uint8Array {
  const hex = mac.replace(/[^0-9a-f]/gi, "");
  if (hex.length !== 12) throw new Error(`not a MAC address: ${mac}`);
  return Uint8Array.from(hex.match(/../g)!.map((b) => parseInt(b, 16)));
}

/**
 * A synthetic (but standards-shaped) 802.11 Data frame carrying one captured EAPOL-Key message: a
 * 24-byte header with the addresses in the direction the message actually travelled, then the LLC/SNAP
 * header, then the EAPOL bytes the device captured (which is exactly what its own parser strips out, so
 * this is the faithful inverse of that parse — see crates/core/src/features/handshake/parser.rs).
 */
function frameBytes(bssid: string, client: string, m: HandshakeMessage): Uint8Array {
  const fromAp = m.message === 1 || m.message === 3; // AP -> client (ANonce messages)
  const [addr1, addr2] = fromAp ? [client, bssid] : [bssid, client];
  const out = new Uint8Array(24 + LLC_SNAP_EAPOL.length + m.eapol.length);
  out[0] = 0x08; // frame control: plain Data, no QoS (not known from the capture)
  out[1] = fromAp ? 0x02 : 0x01; // FromDS or ToDS
  out.set(macBytes(addr1), 4);
  out.set(macBytes(addr2), 10);
  out.set(macBytes(bssid), 16); // address 3: the BSSID either way (EAPOL never leaves the BSS)
  out.set(LLC_SNAP_EAPOL, 24);
  out.set(m.eapol, 24 + LLC_SNAP_EAPOL.length);
  return out;
}

function u32le(out: DataView, offset: number, v: number): void {
  out.setUint32(offset, v, true);
}
function u16le(out: DataView, offset: number, v: number): void {
  out.setUint16(offset, v, true);
}

interface PcapFrameEntry {
  at: number;
  message: number;
  frame: Uint8Array;
}

/**
 * Builds a classic libpcap file containing frames from multiple handshake pairs,
 * ordered chronologically by capture timestamp. Suitable for Wireshark and `hcxpcapngtool`.
 */
export function buildMultiPcap(pairs: readonly HandshakePair[]): Uint8Array<ArrayBuffer> {
  const entries: PcapFrameEntry[] = [];
  for (const pair of pairs) {
    for (const m of pair.messages.values()) {
      entries.push({
        at: m.at,
        message: m.message,
        frame: frameBytes(pair.bssid, pair.client, m),
      });
    }
  }

  // Sort chronologically; break timestamp ties by message sequence (M1, M2, M3, M4)
  entries.sort((a, b) => (a.at - b.at) || (a.message - b.message));

  const total = 24 + entries.reduce((n, e) => n + 16 + e.frame.length, 0);
  const buf = new ArrayBuffer(total);
  const view = new DataView(buf);
  const bytes = new Uint8Array(buf);

  u32le(view, 0, PCAP_MAGIC_LE);
  u16le(view, 4, 2); // version_major
  u16le(view, 6, 4); // version_minor
  u32le(view, 8, 0); // thiszone
  u32le(view, 12, 0); // sigfigs
  u32le(view, 16, 65535); // snaplen
  u32le(view, 20, LINKTYPE_IEEE802_11);

  let o = 24;
  for (const entry of entries) {
    u32le(view, o, Math.floor(entry.at / 1000)); // ts_sec
    u32le(view, o + 4, (entry.at % 1000) * 1000); // ts_usec
    u32le(view, o + 8, entry.frame.length); // incl_len
    u32le(view, o + 12, entry.frame.length); // orig_len
    o += 16;
    bytes.set(entry.frame, o);
    o += entry.frame.length;
  }
  return bytes;
}

/**
 * A classic libpcap file (the `.cap` / `.pcap` extensions name the same format) with one packet per
 * captured message, oldest first. Openable in Wireshark, and the EAPOL content is what tools such as
 * aircrack-ng or hcxpcapngtool read for an offline cracking attempt — subject to any message the device
 * flagged `truncated`, which this file carries over as `origLen` reported equal to the captured length
 * (the true original length past the device's capture cap is not known).
 */
export function buildPcap(pair: HandshakePair): Uint8Array<ArrayBuffer> {
  return buildMultiPcap([pair]);
}

/** `ssid_macwithnocolons`, filesystem-safe, with the client's MAC appended only when it is needed to tell two
 * captures for the same network apart (several clients handshaking with the one AP). `ssid` comes from
 * wherever the network was last seen in a scan (the handshake itself carries no SSID: EAPOL frames don't
 * have one, only beacons do). */
export function captureFilename(pair: HandshakePair, ssid: string, ext: "cap" | "pcap", disambiguateClient = false): string {
  const safe = (s: string) => s.replace(/[^A-Za-z0-9-]+/g, "_").replace(/^_+|_+$/g, "");
  const name = safe(ssid) || "hidden";
  const bssid = pair.bssid.replace(/[^0-9a-f]/gi, "");
  const client = disambiguateClient ? `_${pair.client.replace(/[^0-9a-f]/gi, "")}` : "";
  return `${name}_${bssid}${client}.${ext}`;
}
