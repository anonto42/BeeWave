import { describe, expect, it } from "vitest";
import { alertFromTelemetry, createMonitorKernel, MISSING_CYCLES, WatchlistEngine, WatchlistStore, type WatchEntry } from "../../../src/shared/monitor";
import { ANOMALY_KINDS } from "../../../src/shared/protocol";
import { MockWorld } from "../../../src/shared/transport/mock-world";

const BSSID = "50:c7:bf:aa:bb:01";

describe("anomaly alerts", () => {
  it("maps every anomaly kind to a titled, colour-coded alert", () => {
    const sev = Object.fromEntries(
      ANOMALY_KINDS.map((kind) => [kind, alertFromTelemetry({ type: "anomaly", kind, bssid: BSSID, detail: 27 })!.severity]),
    );
    expect(sev).toEqual({ newBssidForSsid: "warn", twoChannels: "info", rssiOutlier: "info", returned: "info", securityDowngrade: "alert" });
  });

  it("formats security downgrade anomaly with alert severity and clear title", () => {
    const downgrade = alertFromTelemetry({ type: "anomaly", kind: "securityDowngrade", bssid: BSSID, detail: 0 })!;
    expect(downgrade.severity).toBe("alert");
    expect(downgrade.title).toBe("Evil Twin security downgrade detected");
    expect(downgrade.detail).toContain("open or downgraded access point clone");
    expect(downgrade).toMatchObject({ source: "anomaly", bssid: BSSID });
  });

  it("puts the numbers the device sent into the text", () => {
    const outlier = alertFromTelemetry({ type: "anomaly", kind: "rssiOutlier", bssid: BSSID, detail: 27 })!;
    expect(outlier.detail).toContain("27 dB");
    const back = alertFromTelemetry({ type: "anomaly", kind: "returned", bssid: BSSID, detail: 8 })!;
    expect(back.detail).toContain("8 cycles");
    expect(outlier).toMatchObject({ source: "anomaly", bssid: BSSID });
  });

  it("is not raised for ordinary telemetry", () => {
    expect(alertFromTelemetry({ type: "scan", bssid: BSSID, channel: 6, rssi: -50, ssid: "x" })).toBeNull();
    expect(alertFromTelemetry({ type: "status", activeChannel: 1, freeHeap: 1, cpuLoad: 1, dropped: 0, scanning: true })).toBeNull();
  });

  it("the simulated device produces every kind while scanning", () => {
    const world = new MockWorld(5);
    const kinds = new Set<string>();
    let channel = 1;
    for (let i = 0; i < 300; i++) {
      const s = world.step(true, channel, 0);
      channel = s.channel;
      for (const m of s.telemetry) if (m.type === "anomaly") kinds.add(m.kind);
    }
    expect([...kinds].sort()).toEqual([...ANOMALY_KINDS].sort());
  });

  it("the kernel collects them into the chronological feed (newest first)", () => {
    let t = 0;
    const k = createMonitorKernel(() => ++t);
    k.ingest({ type: "anomaly", kind: "twoChannels", bssid: BSSID, detail: 0 });
    k.ingest({ type: "anomaly", kind: "returned", bssid: BSSID, detail: 7 });
    expect(k.alerts.list().map((a) => a.title)).toEqual(["Network came back", "Same BSSID on two channels"]);
  });

  it("formats beacon flood alerts with high severity", () => {
    const alert = alertFromTelemetry({ type: "beaconFloodAlert", rate: 65, uniqueBssids: 42 })!;
    expect(alert).toMatchObject({
      source: "beaconFlood",
      severity: "alert",
      title: "Beacon flood attack: 65/s",
    });
    expect(alert.detail).toContain("42 unique BSSIDs");
  });

  it("formats decloaked alert with revealed SSID and details", () => {
    const alert = alertFromTelemetry({
      type: "decloakedResult",
      bssid: "00:11:22:33:44:55",
      ssid: "SecretCorp",
      channel: 6,
      rssi: -58,
    })!;
    expect(alert).toMatchObject({
      source: "decloaked",
      severity: "warn",
      title: 'Hidden network uncloaked: "SecretCorp"',
    });
    expect(alert.detail).toContain("CH 6");
    expect(alert.detail).toContain("-58 dBm");
  });

  it("formats ble flood alerts with high severity and dominant class", () => {
    const alert = alertFromTelemetry({
      type: "bleFloodAlert",
      rate: 75,
      class: "proximity",
    })!;
    expect(alert).toMatchObject({
      source: "bleFlood",
      severity: "alert",
      title: "BLE advertisement flood: 75/s",
    });
    expect(alert.detail).toContain("Proximity pairing burst");
  });

  it("formats probe flood alerts with high severity", () => {
    const alert = alertFromTelemetry({
      type: "probeFloodAlert",
      rate: 55,
      uniqueClients: 24,
    })!;
    expect(alert).toMatchObject({
      source: "probeFlood",
      severity: "alert",
      title: "Probe request flood attack: 55/s",
    });
    expect(alert.detail).toContain("24 unique client MACs");
  });

  it("formats disassoc flood alerts with warning and alert severity", () => {
    const warnAlert = alertFromTelemetry({
      type: "disassocAlert",
      bssid: BSSID,
      count: 25,
    })!;
    expect(warnAlert).toMatchObject({
      source: "disassoc",
      severity: "warn",
      title: "Disassoc frames: 25/s",
      bssid: BSSID,
    });

    const highAlert = alertFromTelemetry({
      type: "disassocAlert",
      bssid: BSSID,
      count: 65,
    })!;
    expect(highAlert).toMatchObject({
      source: "disassoc",
      severity: "alert",
      title: "Disassoc frames: 65/s",
      bssid: BSSID,
    });
  });
});

describe("watchlist rules (phone only)", () => {
  const pin: WatchEntry = { bssid: BSSID, ssid: "Home" };
  const engine = () => new WatchlistEngine((b) => (b === BSSID ? pin : undefined), () => [pin]);
  const scan = (rssi: number) => ({ bssid: BSSID, ssid: "Home", channel: 6, rssi });

  it("alerts when a pinned network appears, and ignores unpinned ones", () => {
    const e = engine();
    expect(e.observe({ ...scan(-50), bssid: "aa:aa:aa:aa:aa:aa" }, 0)).toEqual([]);
    const [a] = e.observe(scan(-50), 0);
    expect(a).toMatchObject({ source: "watchlist", title: "Watched network appeared", bssid: BSSID });
  });

  it("alerts when it moves by 6 dB or more, not less", () => {
    const e = engine();
    e.observe(scan(-50), 0);
    expect(e.observe(scan(-54), 1)).toEqual([]);
    const [a] = e.observe(scan(-62), 2);
    expect(a.title).toBe("Watched network moved");
    expect(a.detail).toContain("-8 dB");
  });

  it("alerts once when it has been missing for more than the allowed cycles, then again when it returns", () => {
    const e = engine();
    e.observe(scan(-50), 0);
    expect(e.sweep(MISSING_CYCLES)).toEqual([]);
    const [gone] = e.sweep(MISSING_CYCLES + 1);
    expect(gone.title).toBe("Watched network disappeared");
    expect(e.sweep(MISSING_CYCLES + 2)).toEqual([]); // only once
    expect(e.observe(scan(-50), MISSING_CYCLES + 3)[0].title).toBe("Watched network appeared");
  });

  it("stores pins in the browser only and survives blocked storage", () => {
    const data = new Map<string, string>();
    const storage = { getItem: (k: string) => data.get(k) ?? null, setItem: (k: string, v: string) => void data.set(k, v) };
    const a = new WatchlistStore(storage);
    a.add(pin);
    a.add(pin); // no duplicates
    expect(new WatchlistStore(storage).list()).toEqual([pin]);
    a.remove(BSSID);
    expect(new WatchlistStore(storage).list()).toEqual([]);

    const broken = { getItem: () => { throw new Error("blocked"); }, setItem: () => { throw new Error("blocked"); } };
    const b = new WatchlistStore(broken);
    b.add(pin);
    expect(b.list()).toEqual([pin]);
  });

  it("the kernel turns watchlist hits into alerts from the scan stream", () => {
    const k = createMonitorKernel(() => 1, { getItem: () => null, setItem: () => {} });
    k.watchlist.add(pin);
    k.ingest({ type: "scan", bssid: BSSID, channel: 6, rssi: -50, ssid: "Home" });
    expect(k.alerts.list()[0]).toMatchObject({ source: "watchlist", title: "Watched network appeared" });
  });
});
