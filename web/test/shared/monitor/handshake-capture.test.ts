import { describe, expect, it } from "vitest";
import { alertFromTelemetry, createMonitorKernel, exportAll22000, HandshakeState, handshakeToHashcat22000, pairToHex, pmkidToHashcat22000, type HandshakePair } from "../../../src/shared/monitor";
import { MockWorld } from "../../../src/shared/transport/mock-world";

const BSSID = "50:c7:bf:aa:bb:01";
const CLIENT = "6c:5a:72:00:01:02";
const eapol = (n: number) => Uint8Array.from([n, n + 1, n + 2]);

describe("HandshakeState", () => {
  it("is off until told otherwise, and clears captured pairs when switched off", () => {
    const s = new HandshakeState();
    expect(s.enabled).toBe(false);
    s.ingest({ type: "handshakeCaptureState", enabled: true });
    s.ingest({ type: "handshakeFrame", bssid: BSSID, client: CLIENT, message: 1, replayCounter: 1, truncated: false, eapol: eapol(1) });
    expect(s.list()).toHaveLength(1);
    s.ingest({ type: "handshakeCaptureState", enabled: false });
    expect([s.enabled, s.list()]).toEqual([false, []]);
  });

  it("collects messages per pair and marks completion only on HandshakeComplete", () => {
    const s = new HandshakeState();
    s.ingest({ type: "handshakeCaptureState", enabled: true });
    s.ingest({ type: "handshakeFrame", bssid: BSSID, client: CLIENT, message: 1, replayCounter: 1, truncated: false, eapol: eapol(1) });
    expect(s.list()[0].complete).toBe(false);
    s.ingest({ type: "handshakeFrame", bssid: BSSID, client: CLIENT, message: 2, replayCounter: 2, truncated: false, eapol: eapol(2) });
    expect(s.list()[0].messages.size).toBe(2);
    expect(s.list()[0].complete).toBe(false), "the store trusts the device's own completion message, not its own guess";
    s.ingest({ type: "handshakeComplete", bssid: BSSID, client: CLIENT });
    expect(s.list()[0].complete).toBe(true);
  });

  it("keeps different pairs separate and lists the most recently seen first", () => {
    let t = 0;
    const s = new HandshakeState(() => (t += 1));
    s.ingest({ type: "handshakeCaptureState", enabled: true });
    s.ingest({ type: "handshakeFrame", bssid: BSSID, client: CLIENT, message: 1, replayCounter: 1, truncated: false, eapol: eapol(1) });
    s.ingest({ type: "handshakeFrame", bssid: "aa:aa:aa:aa:aa:aa", client: "bb:bb:bb:bb:bb:bb", message: 1, replayCounter: 1, truncated: false, eapol: eapol(9) });
    expect(s.list().map((p) => p.bssid)).toEqual(["aa:aa:aa:aa:aa:aa", BSSID]);
  });

  it("a later frame for the same message replaces it, not the other messages", () => {
    const s = new HandshakeState();
    s.ingest({ type: "handshakeCaptureState", enabled: true });
    s.ingest({ type: "handshakeFrame", bssid: BSSID, client: CLIENT, message: 1, replayCounter: 1, truncated: false, eapol: eapol(1) });
    s.ingest({ type: "handshakeFrame", bssid: BSSID, client: CLIENT, message: 1, replayCounter: 5, truncated: true, eapol: eapol(9) });
    const pair = s.list()[0];
    expect(pair.messages.size).toBe(1);
    expect(pair.messages.get(1)).toMatchObject({ replayCounter: 5, truncated: true });
  });
});

describe("pairToHex", () => {
  it("lists messages in order with hex bytes, flagging a truncated one", () => {
    const s = new HandshakeState();
    s.ingest({ type: "handshakeCaptureState", enabled: true });
    s.ingest({ type: "handshakeFrame", bssid: BSSID, client: CLIENT, message: 2, replayCounter: 1, truncated: false, eapol: eapol(1) });
    s.ingest({ type: "handshakeFrame", bssid: BSSID, client: CLIENT, message: 1, replayCounter: 1, truncated: true, eapol: Uint8Array.from([0xaa, 0xbb]) });
    s.ingest({ type: "handshakeComplete", bssid: BSSID, client: CLIENT });
    const text = pairToHex(s.list()[0]);
    const lines = text.trimEnd().split("\n");
    expect(lines[0]).toBe(`# ${BSSID} <-> ${CLIENT} (complete)`);
    expect(lines[1]).toBe("## message 1 (truncated: longer than the capture cap)");
    expect(lines[2]).toBe("aa bb");
    expect(lines[3]).toBe("## message 2");
    expect(lines[4]).toBe("01 02 03");
  });
});

describe("handshakeComplete alert", () => {
  it("names both sides and lands in the handshake source", () => {
    const a = alertFromTelemetry({ type: "handshakeComplete", bssid: BSSID, client: CLIENT })!;
    expect(a).toMatchObject({ source: "handshake", severity: "info", bssid: BSSID });
    expect(a.detail).toContain(CLIENT);
  });
});

describe("the simulated device produces a capturable handshake only while listening", () => {
  const run = (world: MockWorld, ticks: number) => {
    const out = [];
    let ch = 1;
    for (let i = 0; i < ticks; i++) {
      const s = world.step(true, ch, 0);
      ch = s.channel;
      out.push(...s.telemetry);
    }
    return out;
  };

  it("emits nothing while off", () => {
    const world = new MockWorld(1);
    expect(run(world, 60).filter((m) => m.type.startsWith("handshake"))).toEqual([]);
  });

  it("emits messages 1 and 2 plus completion once switched on, and the kernel reports it as a pair", () => {
    const world = new MockWorld(1);
    world.setHandshakeCapture(true);
    const k = createMonitorKernel();
    k.handshake.ingest({ type: "handshakeCaptureState", enabled: true });
    for (const m of run(world, 40)) k.ingest(m);
    const pairs = k.handshake.list();
    expect(pairs).toHaveLength(1);
    expect(pairs[0].complete).toBe(true);
    expect([...pairs[0].messages.keys()].sort()).toEqual([1, 2]);
    expect(k.alerts.list().some((a) => a.source === "handshake")).toBe(true);
  });
});

describe("PMKID extraction and Hashcat export", () => {
  it("ingests PMKID and dedupes repeated frames", () => {
    const s = new HandshakeState();
    s.ingest({ type: "pmkidFound", bssid: BSSID, client: CLIENT, pmkid: "00112233445566778899aabbccddeeff" });
    s.ingest({ type: "pmkidFound", bssid: BSSID, client: CLIENT, pmkid: "00112233445566778899aabbccddeeff" });
    const records = s.pmkidsFor(BSSID);
    expect(records).toHaveLength(1);
    expect(records[0]).toMatchObject({ bssid: BSSID, client: CLIENT, pmkid: "00112233445566778899aabbccddeeff" });
  });

  it("formats a PMKID record as a Hashcat mode 22000 line", () => {
    const line = pmkidToHashcat22000(
      { bssid: "50:c7:bf:aa:bb:01", client: "6c:5a:72:00:01:02", pmkid: "00112233445566778899aabbccddeeff", at: 1000 },
      "MyNetwork"
    );
    // Format: WPA*01*<pmkid>*<bssid_no_sep>*<client_no_sep>*<essid_hex>***
    // "MyNetwork" in hex: 4d794e6574776f726b
    expect(line).toBe("WPA*01*00112233445566778899aabbccddeeff*50c7bfaabb01*6c5a72000102*4d794e6574776f726b***");
  });
});

describe("handshakeToHashcat22000", () => {
  const createFrame = (len: number, fillByte: number) => {
    const b = new Uint8Array(len);
    b.fill(fillByte);
    return b;
  };

  it("formats a 4-way handshake M1+M2 as Hashcat mode 22000 WPA*02 line with zeroed MIC", () => {
    const m1Bytes = createFrame(60, 0x11);
    // Fill ANonce (32 bytes at offset 17..49)
    m1Bytes.fill(0xaa, 17, 49);

    const m2Bytes = createFrame(100, 0x22);
    // Fill MIC (16 bytes at offset 81..97)
    m2Bytes.fill(0xbb, 81, 97);

    const pair: HandshakePair = {
      bssid: "50:c7:bf:aa:bb:01",
      client: "6c:5a:72:00:01:02",
      complete: true,
      firstSeen: 1000,
      lastSeen: 1000,
      messages: new Map([
        [1, { message: 1, replayCounter: 1, truncated: false, eapol: m1Bytes, at: 1000 }],
        [2, { message: 2, replayCounter: 2, truncated: false, eapol: m2Bytes, at: 1000 }],
      ]),
    };

    const line = handshakeToHashcat22000(pair, "MyNetwork");
    expect(line).not.toBeNull();
    const parts = line!.split("*");

    expect(parts[0]).toBe("WPA");
    expect(parts[1]).toBe("02");
    // MIC: 16 bytes of 0xbb = 32 hex chars of "bb"
    expect(parts[2]).toBe("bb".repeat(16));
    // AP MAC
    expect(parts[3]).toBe("50c7bfaabb01");
    // STA MAC
    expect(parts[4]).toBe("6c5a72000102");
    // ESSID hex
    expect(parts[5]).toBe("4d794e6574776f726b");
    // ANonce: 32 bytes of 0xaa = 64 hex chars of "aa"
    expect(parts[6]).toBe("aa".repeat(32));
    // EAPOL: 100 bytes = 200 hex chars. Bytes 81..97 must be zeroed ("00")!
    const eapolHex = parts[7];
    expect(eapolHex.length).toBe(200);
    expect(eapolHex.slice(81 * 2, 97 * 2)).toBe("00".repeat(16));
    // Non-MIC bytes should remain 0x22
    expect(eapolHex.slice(0, 10)).toBe("22".repeat(5));
    // Message pair flag for M1+M2
    expect(parts[8]).toBe("00");
  });

  it("supports M3+M2 fallback and sets message_pair to 02", () => {
    const m3Bytes = createFrame(60, 0x33);
    m3Bytes.fill(0xcc, 17, 49); // ANonce in M3

    const m2Bytes = createFrame(100, 0x44);
    m2Bytes.fill(0xdd, 81, 97); // MIC in M2

    const pair: HandshakePair = {
      bssid: "11:22:33:44:55:66",
      client: "aa:bb:cc:dd:ee:ff",
      complete: true,
      firstSeen: 1000,
      lastSeen: 1000,
      messages: new Map([
        [3, { message: 3, replayCounter: 1, truncated: false, eapol: m3Bytes, at: 1000 }],
        [2, { message: 2, replayCounter: 2, truncated: false, eapol: m2Bytes, at: 1000 }],
      ]),
    };

    const line = handshakeToHashcat22000(pair, "TestWifi");
    expect(line).not.toBeNull();
    const parts = line!.split("*");
    expect(parts[1]).toBe("02");
    expect(parts[2]).toBe("dd".repeat(16));
    expect(parts[6]).toBe("cc".repeat(32));
    expect(parts[8]).toBe("02"); // M3+M2 message_pair is 02
  });

  it("returns null if Message 2 is missing or Message 1/3 is missing", () => {
    const m1Bytes = createFrame(60, 0x11);
    const pairNoM2: HandshakePair = {
      bssid: BSSID,
      client: CLIENT,
      complete: false,
      firstSeen: 1000,
      lastSeen: 1000,
      messages: new Map([
        [1, { message: 1, replayCounter: 1, truncated: false, eapol: m1Bytes, at: 1000 }],
      ]),
    };
    expect(handshakeToHashcat22000(pairNoM2, "Wifi")).toBeNull();

    const m2Bytes = createFrame(100, 0x22);
    const pairNoM1: HandshakePair = {
      bssid: BSSID,
      client: CLIENT,
      complete: false,
      firstSeen: 1000,
      lastSeen: 1000,
      messages: new Map([
        [2, { message: 2, replayCounter: 2, truncated: false, eapol: m2Bytes, at: 1000 }],
      ]),
    };
    expect(handshakeToHashcat22000(pairNoM1, "Wifi")).toBeNull();
  });

  it("returns null if frames are too short to extract Nonce or MIC", () => {
    const pairShortM1: HandshakePair = {
      bssid: BSSID,
      client: CLIENT,
      complete: true,
      firstSeen: 1000,
      lastSeen: 1000,
      messages: new Map([
        [1, { message: 1, replayCounter: 1, truncated: false, eapol: createFrame(30, 0x11), at: 1000 }], // < 49 bytes
        [2, { message: 2, replayCounter: 2, truncated: false, eapol: createFrame(100, 0x22), at: 1000 }],
      ]),
    };
    expect(handshakeToHashcat22000(pairShortM1, "Wifi")).toBeNull();

    const pairShortM2: HandshakePair = {
      bssid: BSSID,
      client: CLIENT,
      complete: true,
      firstSeen: 1000,
      lastSeen: 1000,
      messages: new Map([
        [1, { message: 1, replayCounter: 1, truncated: false, eapol: createFrame(60, 0x11), at: 1000 }],
        [2, { message: 2, replayCounter: 2, truncated: false, eapol: createFrame(80, 0x22), at: 1000 }], // < 97 bytes
      ]),
    };
    expect(handshakeToHashcat22000(pairShortM2, "Wifi")).toBeNull();
  });
});

describe("allPmkids", () => {
  it("returns all PMKIDs across all BSSIDs, newest first", () => {
    let t = 1000;
    const s = new HandshakeState(() => t);
    t = 1100;
    s.ingest({ type: "pmkidFound", bssid: "11:22:33:44:55:66", client: CLIENT, pmkid: "00112233445566778899aabbccddeeff" });
    t = 1200;
    s.ingest({ type: "pmkidFound", bssid: "aa:bb:cc:dd:ee:ff", client: CLIENT, pmkid: "ffeeddccbbaa99887766554433221100" });

    const all = s.allPmkids();
    expect(all).toHaveLength(2);
    expect(all[0].bssid).toBe("aa:bb:cc:dd:ee:ff");
    expect(all[1].bssid).toBe("11:22:33:44:55:66");
  });
});

describe("exportAll22000", () => {
  const createFrame = (len: number, fillByte: number) => {
    const b = new Uint8Array(len);
    b.fill(fillByte);
    return b;
  };

  it("returns empty string when no pairs and no PMKIDs are present", () => {
    expect(exportAll22000([], [])).toBe("");
  });

  it("skips non-crackable pairs and returns empty string", () => {
    const pairIncomplete: HandshakePair = {
      bssid: BSSID,
      client: CLIENT,
      complete: false,
      firstSeen: 1000,
      lastSeen: 1000,
      messages: new Map([
        [1, { message: 1, replayCounter: 1, truncated: false, eapol: createFrame(60, 0x11), at: 1000 }],
      ]),
    };
    expect(exportAll22000([pairIncomplete], [])).toBe("");
  });

  it("consolidates valid handshakes (WPA*02*) and PMKIDs (WPA*01*) into multi-line 22000 output", () => {
    const m1Bytes = createFrame(60, 0x11);
    m1Bytes.fill(0xaa, 17, 49);
    const m2Bytes = createFrame(100, 0x22);
    m2Bytes.fill(0xbb, 81, 97);

    const pair: HandshakePair = {
      bssid: "50:c7:bf:aa:bb:01",
      client: "6c:5a:72:00:01:02",
      complete: true,
      firstSeen: 1000,
      lastSeen: 1000,
      messages: new Map([
        [1, { message: 1, replayCounter: 1, truncated: false, eapol: m1Bytes, at: 1000 }],
        [2, { message: 2, replayCounter: 2, truncated: false, eapol: m2Bytes, at: 1000 }],
      ]),
    };

    const pmkid = {
      bssid: "60:c7:bf:aa:bb:02",
      client: "7c:5a:72:00:01:03",
      pmkid: "00112233445566778899aabbccddeeff",
      at: 1000,
    };

    const ssids: Record<string, string> = {
      "50:c7:bf:aa:bb:01": "OfficeNet",
      "60:c7:bf:aa:bb:02": "GuestNet",
    };

    const result = exportAll22000([pair], [pmkid], (bssid) => ssids[bssid] ?? "");
    const lines = result.trim().split("\n");
    expect(lines).toHaveLength(2);

    // First line should be WPA*02* for the handshake
    expect(lines[0].startsWith("WPA*02*")).toBe(true);
    expect(lines[0]).toContain("50c7bfaabb01");
    // "OfficeNet" in hex: 4f66666963654e6574
    expect(lines[0]).toContain("4f66666963654e6574");

    // Second line should be WPA*01* for the PMKID
    expect(lines[1].startsWith("WPA*01*")).toBe(true);
    expect(lines[1]).toContain("00112233445566778899aabbccddeeff");
    expect(lines[1]).toContain("60c7bfaabb02");
    // "GuestNet" in hex: 47756573744e6574
    expect(lines[1]).toContain("47756573744e6574");
  });
});
