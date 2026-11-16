import { describe, expect, it } from "vitest";
import vectors from "../vectors.json";
import { decodeTelemetry, encodeCommand, type Command } from "../../src/shared/protocol";

const bytes = (h: string) => Uint8Array.from(h.match(/../g)!.map((x) => parseInt(x, 16)));
const hex = (b: Uint8Array) => [...b].map((x) => x.toString(16).padStart(2, "0")).join("");

describe("telemetry decode (golden vectors from Rust)", () => {
  it("status", () => {
    expect(decodeTelemetry(bytes(vectors.telemetry[0].hex)).msg).toEqual({ type: "status", activeChannel: 6, freeHeap: 123456, cpuLoad: 42, dropped: 300 });
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
  ];
  cmds.forEach((c, i) => it(c.type, () => expect(hex(encodeCommand(i, c))).toBe(vectors.commands[i].hex)));
});
