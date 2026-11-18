import { describe, expect, it } from "vitest";
import { alertFromTelemetry, createMonitorKernel, HandshakeState, pairToHex, pmkidToHashcat22000 } from "../../../src/shared/monitor";
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
