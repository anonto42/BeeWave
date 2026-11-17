import { describe, expect, it } from "vitest";
import { alertFromTelemetry, createMonitorKernel, DEAUTH_FLOOD } from "../../../src/shared/monitor";
import { MockWorld } from "../../../src/shared/transport/mock-world";

const bursts = (world: MockWorld, ticks = 600) => {
  let channel = 1;
  const out: number[] = [];
  for (let i = 0; i < ticks; i++) {
    const s = world.step(true, channel, 0);
    channel = s.channel;
    for (const m of s.telemetry) if (m.type === "deauth") out.push(m.count);
  }
  return out;
};

describe("deauth alert threshold", () => {
  it("only alerts at or above the configured threshold", () => {
    const world = new MockWorld(3);
    world.setMonitor({ deauthThreshold: 30, bleEnabled: false, bleSpikeFactor: 3 });
    const counts = bursts(world);
    expect(counts.length).toBeGreaterThan(0);
    expect(Math.min(...counts)).toBeGreaterThanOrEqual(30);
  });

  it("a lower threshold alerts on more bursts than a higher one", () => {
    const low = new MockWorld(3);
    low.setMonitor({ deauthThreshold: 10, bleEnabled: false, bleSpikeFactor: 3 });
    const high = new MockWorld(3);
    high.setMonitor({ deauthThreshold: 60, bleEnabled: false, bleSpikeFactor: 3 });
    expect(bursts(low).length).toBeGreaterThan(bursts(high).length);
  });

  it("a threshold of 0 turns the detector off", () => {
    const world = new MockWorld(3);
    world.setMonitor({ deauthThreshold: 0, bleEnabled: false, bleSpikeFactor: 3 });
    expect(bursts(world)).toEqual([]);
  });

  it("nothing is simulated while not scanning", () => {
    const world = new MockWorld(3);
    for (let i = 0; i < 100; i++) expect(world.step(false, 1, 0).telemetry).toEqual([]);
  });

  it("the device refuses a bad BLE factor and keeps the old config", () => {
    const world = new MockWorld(3);
    const reply = world.setMonitor({ deauthThreshold: 99, bleEnabled: true, bleSpikeFactor: 99 });
    expect(reply).toMatchObject({ type: "event", code: 5 });
    expect(world.config.deauthThreshold).toBe(10);
  });

  it("alerts are yellow below the flood size and red from it", () => {
    const a = (count: number) => alertFromTelemetry({ type: "deauth", bssid: "50:c7:bf:aa:bb:01", count })!;
    expect(a(DEAUTH_FLOOD - 1).severity).toBe("warn");
    expect(a(DEAUTH_FLOOD).severity).toBe("alert");
    expect(a(12)).toMatchObject({ source: "deauth", bssid: "50:c7:bf:aa:bb:01" });
    expect(a(12).detail).toContain("TP-Link");
  });

  it("the kernel keeps a per-BSSID series and raises an alert for each burst", () => {
    const k = createMonitorKernel(() => 5);
    for (const count of [12, 30, 70]) k.ingest({ type: "deauth", bssid: "aa:bb:cc:dd:ee:ff", count });
    const src = k.state.deauth.get("aa:bb:cc:dd:ee:ff")!;
    expect([src.last, src.alerts, src.series]).toEqual([70, 3, [12, 30, 70]]);
    expect(k.alerts.list().map((a) => a.severity)).toEqual(["alert", "warn", "warn"]); // newest first
  });
});
