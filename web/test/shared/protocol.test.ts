import { describe, expect, it } from "vitest";
import vectors from "../vectors.json";
import { decodeTelemetry, encodeCommand, type Command } from "../../src/shared/protocol";

const bytes = (h: string) => Uint8Array.from(h.match(/../g)!.map((x) => parseInt(x, 16)));
const hex = (b: Uint8Array) => [...b].map((x) => x.toString(16).padStart(2, "0")).join("");

describe("telemetry decode (golden vectors from Rust)", () => {
  it("status", () => {
    expect(decodeTelemetry(bytes(vectors.telemetry[0].hex)).msg).toEqual({ type: "status", activeChannel: 6, freeHeap: 123456, cpuLoad: 42, dropped: 300, scanning: true });
  });
  it("scan result with utf8 + negative rssi", () => {
    expect(decodeTelemetry(bytes(vectors.telemetry[1].hex))).toEqual({ seq: 1, msg: { type: "scan", bssid: "01:02:03:04:05:06", channel: 11, rssi: -67, ssid: "Café-WiFi" } });
  });
  it("settings", () => {
    expect(decodeTelemetry(bytes(vectors.telemetry[4].hex)).msg).toEqual({ type: "settings", channel: 6, autoScan: true });
  });
  it("event + otaAck", () => {
    expect(decodeTelemetry(bytes(vectors.telemetry[2].hex)).msg).toEqual({ type: "event", code: 7, message: "hello" });
    expect(decodeTelemetry(bytes(vectors.telemetry[3].hex)).msg).toEqual({ type: "otaAck", nextOffset: 4096 });
  });
  it("auth challenge + result", () => {
    expect(decodeTelemetry(bytes(vectors.telemetry[5].hex)).msg).toEqual({ type: "authChallenge", nonce: new Uint8Array(16).fill(7) });
    expect(decodeTelemetry(bytes(vectors.telemetry[6].hex)).msg).toEqual({ type: "authResult", ok: true });
    expect(decodeTelemetry(bytes(vectors.telemetry[7].hex)).msg).toEqual({ type: "unclaimed" });
  });
  it("monitoring messages", () => {
    const m = (i: number) => decodeTelemetry(bytes(vectors.telemetry[i].hex)).msg;
    expect(m(8)).toEqual({ type: "deauth", bssid: "01:02:03:04:05:06", count: 42 });
    expect(m(9)).toEqual({ type: "bleStats", fastPair: 1, ibeacon: 2, proximity: 300, generic: 4000 });
    expect(m(10)).toEqual({ type: "bleSpike", class: "proximity", count: 500, baseline: 40 });
    expect(m(11)).toEqual({ type: "anomaly", kind: "rssiOutlier", bssid: "09:08:07:06:05:04", detail: 27 });
    expect(m(12)).toEqual({ type: "monitorConfig", deauthThreshold: 10, bleEnabled: true, bleSpikeFactor: 3 });
  });
  it("settings management messages", () => {
    const m = (i: number) => decodeTelemetry(bytes(vectors.telemetry[i].hex)).msg;
    expect(m(13)).toEqual({
      type: "settingsAll",
      schema: 1,
      revision: 300,
      settings: {
        scan: { scanAtBoot: true, scanIntervalMs: 500, dedupRssiDb: 6, ssidFilter: "Cafe", channelFilterMask: 0b1_0000_0100_0001 },
        radio: { defaultChannel: 0, hopDwellMs: 250, hopOrder: "custom", customHopList: [1, 6, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
        telemetry: { statusPeriodMs: 1000, logLevel: "info" },
        alerts: { deauthThreshold: 10, deauthWindowMs: 1000 },
      },
    });
    expect(m(14)).toEqual({ type: "settingsSaved", revision: 7, ok: false, reason: "windowTooShort" });
    expect(m(15)).toEqual({ type: "settingsSaved", revision: 8, ok: true, reason: null });
    expect(m(16)).toEqual({
      type: "storageInfo", nvsUsed: 3000, nvsFree: 21576, ota0Active: true, ota1Active: false, ota0Size: 1572864, ota1Size: 1572864, settingsRevision: 9,
    });
    expect(m(17)).toEqual({ type: "selfTest", radio: true, nvs: true, ble: false, otaSlot: true, busy: false });
    expect(m(18)).toEqual({
      type: "deviceInfo", version: "0.1.0", gitSha: "a1b2c3d", built: "2026-11-18T10:00:00Z", otaKeyFp: "ab".repeat(16),
      requestConnParams: false, coexDebug: false, bonding: false,
    });
    expect(m(19)).toEqual({ type: "rejected", reason: "busy" });
  });
  it("rejects an unknown enum variant", () => {
    // BleSpike with class index 9
    expect(() => decodeTelemetry(bytes("010a0a09f40328"))).toThrow(/unknown enum variant/);
  });
  it("rejects bad version / truncated", () => {
    expect(() => decodeTelemetry(Uint8Array.from([9, 0, 0]))).toThrow();
    expect(() => decodeTelemetry(bytes("0101"))).toThrow();
  });
});

describe("command encode (matches Rust bytes)", () => {
  const cmds: Command[] = [
    { type: "startScan", bandMask: 3 },
    { type: "stop" },
    { type: "configureRadio", channel: 13 },
    { type: "reboot" },
    { type: "otaBegin", size: 1_000_000, sha256: new Uint8Array(32).fill(9) },
    { type: "otaChunk", offset: 96, data: Uint8Array.from([1, 2, 3]) },
    { type: "otaEnd" },
    { type: "getSettings" },
    { type: "setSettings", channel: 0, autoScan: true },
    { type: "authBegin" },
    { type: "authProve", mac: new Uint8Array(32).fill(5) },
    { type: "otaEndSigned", signature: Uint8Array.from([...new Uint8Array(32).fill(3), ...new Uint8Array(32).fill(4)]) },
    { type: "provision", key: new Uint8Array(32).fill(6) },
    { type: "getMonitor" },
    { type: "setMonitor", deauthThreshold: 10, bleEnabled: true, bleSpikeFactor: 3 },
    { type: "getSettingsAll" },
    {
      type: "saveSettingsAll",
      settings: {
        scan: { scanAtBoot: true, scanIntervalMs: 500, dedupRssiDb: 6, ssidFilter: "Cafe", channelFilterMask: 0b1_0000_0100_0001 },
        radio: { defaultChannel: 0, hopDwellMs: 250, hopOrder: "custom", customHopList: [1, 6, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
        telemetry: { statusPeriodMs: 1000, logLevel: "info" },
        alerts: { deauthThreshold: 10, deauthWindowMs: 1000 },
      },
    },
    { type: "factoryReset" },
    { type: "runSelfTest" },
    { type: "getStorageInfo" },
    { type: "getDeviceInfo" },
  ];
  cmds.forEach((c, i) => it(c.type, () => expect(hex(encodeCommand(i, c))).toBe(vectors.commands[i].hex)));
});
