import { describe, expect, it } from "vitest";
import { AD_CLASSES } from "../../../src/shared/protocol";
import { alertFromTelemetry, createMonitorKernel, setMonitorCommand, BLE_FLOOD_FACTOR } from "../../../src/shared/monitor";
import { MockWorld } from "../../../src/shared/transport/mock-world";

/**
 * Classification itself runs on the device (crates/core ble_monitor, with its own host tests). Here: the
 * class names match the wire order, the counts per class reach the UI, and the spike rule is honoured.
 */
const run = (world: MockWorld, ticks: number) => {
  const out = [];
  for (let i = 0; i < ticks; i++) out.push(...world.step(false, 1, 0).telemetry);
  return out;
};

describe("BLE advertisement classes", () => {
  it("the class names follow the wire (variant) order", () => {
    expect([...AD_CLASSES]).toEqual(["fastPair", "ibeacon", "proximity", "generic", "findMy"]);
  });

  it("nothing is reported while the BLE monitor is off", () => {
    expect(run(new MockWorld(1), 200)).toEqual([]);
  });

  it("per-class counts reach the monitor state and build a series", () => {
    const world = new MockWorld(1);
    world.setMonitor({ deauthThreshold: 10, bleEnabled: true, bleSpikeFactor: 3 });
    const k = createMonitorKernel();
    for (const m of run(world, 100)) k.ingest(m);
    expect(k.state.bleLatest).not.toBeNull();
    expect(k.state.bleSeries.generic.length).toBe(10); // one BleStats every 10 s
    expect(k.state.bleLatest!.generic).toBeGreaterThan(k.state.bleLatest!.ibeacon);
  });

  it("raises a spike only above max(baseline x factor, baseline + 20)", () => {
    const spikes = (factor: number) => {
      const world = new MockWorld(1);
      world.setMonitor({ deauthThreshold: 10, bleEnabled: true, bleSpikeFactor: factor });
      return run(world, 130).filter((m) => m.type === "bleSpike");
    };
    expect(spikes(3).length).toBeGreaterThan(0);
    expect(spikes(20)).toEqual([]); // 90 is not above 8 x 20
  });

  it("alerts name the class and turn red at a flood", () => {
    const spike = (count: number) => alertFromTelemetry({ type: "bleSpike", class: "proximity", count, baseline: 8 })!;
    expect(spike(40)).toMatchObject({ source: "ble", severity: "warn" });
    expect(spike(40).title).toContain("Proximity pairing");
    expect(spike(8 * BLE_FLOOD_FACTOR).severity).toBe("alert");
  });

  it("alerts name Apple Find My (AirTag) on spike", () => {
    const spike = alertFromTelemetry({ type: "bleSpike", class: "findMy", count: 35, baseline: 5 })!;
    expect(spike).toMatchObject({ source: "ble", severity: "warn" });
    expect(spike.title).toContain("Apple Find My (AirTag)");
  });

  it("switching the monitor clears the old BLE history", () => {
    const k = createMonitorKernel();
    k.ingest({ type: "monitorConfig", deauthThreshold: 10, bleEnabled: true, bleSpikeFactor: 3 });
    k.ingest({ type: "bleStats", fastPair: 1, ibeacon: 2, proximity: 3, generic: 4, findMy: 5 });
    expect(k.state.bleLatest).not.toBeNull();
    k.ingest({ type: "monitorConfig", deauthThreshold: 10, bleEnabled: false, bleSpikeFactor: 3 });
    expect(k.state.bleLatest).toBeNull();
  });

  it("builds the full setMonitor command from the current config plus a change", () => {
    expect(setMonitorCommand({ deauthThreshold: 10, bleEnabled: false, bleSpikeFactor: 3 }, { bleEnabled: true })).toEqual({
      type: "setMonitor",
      deauthThreshold: 10,
      bleEnabled: true,
      bleSpikeFactor: 3,
    });
  });
});
