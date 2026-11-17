import { describe, expect, it } from "vitest";
import { describeEvent } from "../../src/features/alerts/describe";
import { controlState } from "../../src/features/control/enabled";
import { signalBars } from "../../src/features/scanner/signal";
import { sortNetworks } from "../../src/features/scanner/sort";
import type { ScanResult } from "../../src/features/scanner/types";

const net = (ssid: string, channel: number, rssi: number): ScanResult => ({ type: "scan", bssid: `00:00:00:00:00:${channel}${rssi}`, ssid, channel, rssi });
const nets = [net("b", 6, -70), net("", 1, -40), net("a", 11, -55)];

describe("describeEvent", () => {
  it("titles the device's error codes", () => {
    expect(describeEvent(4, "not authenticated")).toBe("Locked: not authenticated");
    expect(describeEvent(1, "")).toBe("Radio error");
  });
  it("ignores informational events", () => {
    expect(describeEvent(100, "ack startScan")).toBeNull();
  });
});

describe("controlState", () => {
  it("flips the one scan button between start and stop once authenticated", () => {
    expect(controlState("connected", "authenticated", false)).toEqual({ mode: "IDLE", run: { enabled: true, label: "START SCAN" }, channelEnabled: true });
    expect(controlState("connected", "authenticated", true)).toMatchObject({ mode: "SCANNING", run: { label: "STOP SCAN", enabled: true } });
  });
  it("keeps the controls off until the key handshake succeeded, and says why", () => {
    expect(controlState("connected", "authenticating", false)).toMatchObject({ mode: "AUTHENTICATING", channelEnabled: false, run: { enabled: false } });
    expect(controlState("connected", "no-key", false).mode).toBe("LOCKED");
    expect(controlState("connected", "failed", false).mode).toBe("LOCKED");
  });
  it("is off while offline", () => {
    expect(controlState("disconnected", "idle", false)).toMatchObject({ mode: "OFFLINE", channelEnabled: false, run: { enabled: false } });
    expect(controlState("connecting", "idle", false).mode).toBe("OFFLINE");
  });
});

describe("signalBars", () => {
  it("maps RSSI to 1-4 bars", () => {
    expect([-40, -55, -56, -67, -70, -77, -90].map(signalBars)).toEqual([4, 4, 3, 3, 2, 2, 1]);
  });
});

describe("sortNetworks", () => {
  it("sorts by signal, channel and name without mutating the input", () => {
    expect(sortNetworks(nets, "rssi").map((n) => n.rssi)).toEqual([-40, -55, -70]);
    expect(sortNetworks(nets, "channel").map((n) => n.channel)).toEqual([1, 6, 11]);
    expect(sortNetworks(nets, "ssid").map((n) => n.ssid)).toEqual(["a", "b", ""]);
    expect(nets[0].ssid).toBe("b");
  });
});
