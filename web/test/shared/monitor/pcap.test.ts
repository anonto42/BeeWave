import { describe, expect, it } from "vitest";
import { buildPcap, captureFilename, HandshakeState } from "../../../src/shared/monitor";

const BSSID = "50:c7:bf:aa:bb:01";
const CLIENT = "6c:5a:72:00:01:02";

function completedPair(eapol1 = Uint8Array.from([1, 2, 3]), eapol2 = Uint8Array.from([4, 5, 6, 7])) {
  const s = new HandshakeState(() => 1_700_000_000_123);
  s.ingest({ type: "handshakeCaptureState", enabled: true });
  s.ingest({ type: "handshakeFrame", bssid: BSSID, client: CLIENT, message: 1, replayCounter: 1, truncated: false, eapol: eapol1 });
  s.ingest({ type: "handshakeFrame", bssid: BSSID, client: CLIENT, message: 2, replayCounter: 2, truncated: false, eapol: eapol2 });
  s.ingest({ type: "handshakeComplete", bssid: BSSID, client: CLIENT });
  return s.list()[0];
}

function readPcap(bytes: Uint8Array) {
  const v = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const header = { magic: v.getUint32(0, true), linktype: v.getUint32(20, true) };
  const packets: Array<{ len: number; bytes: Uint8Array }> = [];
  let o = 24;
  while (o < bytes.length) {
    const inclLen = v.getUint32(o + 8, true);
    const origLen = v.getUint32(o + 12, true);
    expect(origLen).toBe(inclLen); // the true original length past the capture cap is never known
    packets.push({ len: inclLen, bytes: bytes.slice(o + 16, o + 16 + inclLen) });
    o += 16 + inclLen;
  }
  return { header, packets };
}

describe("buildPcap", () => {
  it("writes a valid classic pcap header (LE magic, IEEE 802.11 linktype)", () => {
    const { header } = readPcap(buildPcap(completedPair()));
    expect(header).toEqual({ magic: 0xa1b2c3d4, linktype: 105 });
  });

  it("writes one 802.11 Data frame per captured message, oldest message first", () => {
    const { packets } = readPcap(buildPcap(completedPair()));
    expect(packets).toHaveLength(2);
    // header(24) + LLC/SNAP(8) + eapol bytes
    expect(packets[0].len).toBe(24 + 8 + 3);
    expect(packets[1].len).toBe(24 + 8 + 4);
  });

  it("puts the LLC/SNAP + EtherType 0x888E right after a 24-byte 802.11 header, then the exact EAPOL bytes back", () => {
    const eapol = Uint8Array.from([0xde, 0xad, 0xbe, 0xef]);
    const { packets } = readPcap(buildPcap(completedPair(eapol)));
    const f = packets[0].bytes;
    expect([...f.slice(24, 32)]).toEqual([0xaa, 0xaa, 0x03, 0x00, 0x00, 0x00, 0x88, 0x8e]);
    expect([...f.slice(32)]).toEqual([...eapol]);
  });

  it("directs message 1 from the AP and message 2 from the client (FromDS vs ToDS)", () => {
    const { packets } = readPcap(buildPcap(completedPair()));
    const bssidBytes = [0x50, 0xc7, 0xbf, 0xaa, 0xbb, 0x01];
    const clientBytes = [0x6c, 0x5a, 0x72, 0x00, 0x01, 0x02];
    const msg1 = packets[0].bytes;
    expect(msg1[1] & 0x03).toBe(0x02); // FromDS
    expect([...msg1.slice(4, 10)]).toEqual(clientBytes); // addr1 = receiver = client
    expect([...msg1.slice(10, 16)]).toEqual(bssidBytes); // addr2 = transmitter = AP
    const msg2 = packets[1].bytes;
    expect(msg2[1] & 0x03).toBe(0x01); // ToDS
    expect([...msg2.slice(4, 10)]).toEqual(bssidBytes); // addr1 = receiver = AP
    expect([...msg2.slice(10, 16)]).toEqual(clientBytes); // addr2 = transmitter = client
  });

  it("address 3 is always the BSSID", () => {
    const { packets } = readPcap(buildPcap(completedPair()));
    const bssidBytes = [0x50, 0xc7, 0xbf, 0xaa, 0xbb, 0x01];
    for (const p of packets) expect([...p.bytes.slice(16, 22)]).toEqual(bssidBytes);
  });
});

describe("captureFilename", () => {
  it("combines the SSID and the BSSID, filesystem-safe", () => {
    const p = completedPair();
    expect(captureFilename(p, "Coffee Shop!", "pcap")).toBe("Coffee_Shop_50c7bfaabb01.pcap");
    expect(captureFilename(p, "Coffee Shop!", "cap")).toBe("Coffee_Shop_50c7bfaabb01.cap");
  });

  it("falls back to 'hidden' for a blank SSID", () => {
    expect(captureFilename(completedPair(), "", "pcap")).toBe("hidden_50c7bfaabb01.pcap");
  });

  it("adds the client's MAC only when asked to disambiguate", () => {
    const p = completedPair();
    expect(captureFilename(p, "Home", "pcap")).toBe("Home_50c7bfaabb01.pcap");
    expect(captureFilename(p, "Home", "pcap", true)).toBe("Home_50c7bfaabb01_6c5a72000102.pcap");
  });
});
