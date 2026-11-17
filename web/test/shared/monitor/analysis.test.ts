import { describe, expect, it } from "vitest";
import { channelActivity, channelOccupancy, diffSnapshots, sparklinePoints } from "../../../src/shared/monitor";
import { vendorOf } from "../../../src/shared/oui";

const n = (bssid: string, channel: number, rssi: number, ssid = "x") => ({ bssid, ssid, channel, rssi });

describe("channel statistics", () => {
  it("counts access points per channel 1-13 and ignores anything else", () => {
    const occ = channelOccupancy([{ channel: 1 }, { channel: 1 }, { channel: 6 }, { channel: 13 }, { channel: 14 }, { channel: 0 }]);
    expect(occ).toHaveLength(13);
    expect([occ[0], occ[5], occ[12]]).toEqual([2, 1, 1]);
    expect(occ.reduce((a, b) => a + b)).toBe(4);
  });

  it("counts only the reports of the last minute", () => {
    const ev = (at: number, channel: number) => ({ at, bssid: "a", ssid: "", channel, rssi: -50 });
    const act = channelActivity([ev(0, 1), ev(50_000, 6), ev(70_000, 6), ev(95_000, 11)], 100_000);
    // the report at t=0 is 100 s old and aged out; the 50 s, 70 s and 95 s ones are inside the minute
    expect([act[0], act[5], act[10]]).toEqual([0, 2, 1]);
  });
});

describe("sparklinePoints", () => {
  it("scales values into the box, highest at the top", () => {
    const pts = sparklinePoints([-80, -50, -65], 100, 20).split(" ");
    expect(pts).toHaveLength(3);
    const ys = pts.map((p) => Number(p.split(",")[1]));
    expect(ys[1]).toBeLessThan(ys[2]);
    expect(ys[2]).toBeLessThan(ys[0]);
    expect(pts[0].startsWith("0.0,")).toBe(true);
    expect(pts[2].startsWith("100.0,")).toBe(true);
  });

  it("handles empty, single and constant series", () => {
    expect(sparklinePoints([], 100, 20)).toBe("");
    expect(sparklinePoints([-50], 100, 20)).not.toBe("");
    expect(sparklinePoints([-50, -50], 100, 20)).toMatch(/^0\.0,\S+ 100\.0,\S+$/);
  });
});

describe("scan comparison", () => {
  const a = { networks: [n("1", 6, -50), n("2", 1, -60), n("3", 11, -70)] };
  const b = { networks: [n("1", 6, -58), n("2", 3, -61), n("4", 9, -80)] };

  it("finds new, disappeared, RSSI-changed and channel-changed networks", () => {
    const d = diffSnapshots(a, b);
    expect(d.added.map((x) => x.bssid)).toEqual(["4"]);
    expect(d.removed.map((x) => x.bssid)).toEqual(["3"]);
    expect(d.rssiChanged.map((x) => [x.after.bssid, x.delta])).toEqual([["1", -8]]);
    expect(d.channelChanged.map((x) => [x.after.bssid, x.before.channel, x.after.channel])).toEqual([["2", 1, 3]]);
  });

  it("is empty for identical scans", () => {
    expect(diffSnapshots(a, a)).toEqual({ added: [], removed: [], rssiChanged: [], channelChanged: [] });
  });
});

describe("vendor lookup", () => {
  it("knows a few prefixes, in any common notation", () => {
    expect(vendorOf("3c:71:bf:0f:1b:f4")).toBe("Espressif");
    expect(vendorOf("3C-71-BF-0F-1B-F4")).toBe("Espressif");
    expect(vendorOf("b827eb000000")).toBe("Raspberry Pi");
  });
  it("labels random (locally administered) addresses and unknown prefixes honestly", () => {
    expect(vendorOf("02:11:22:33:44:55")).toBe("Randomized");
    expect(vendorOf("00:00:00:11:22:33")).toBe("Unknown");
    expect(vendorOf("zz")).toBe("Unknown");
  });
});
