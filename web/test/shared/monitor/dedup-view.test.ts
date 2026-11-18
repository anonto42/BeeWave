import { describe, expect, it } from "vitest";
import { ScanStore, deltaView, lastDelta, RSSI_DELTA_DB } from "../../../src/shared/monitor";
import { MockWorld } from "../../../src/shared/transport/mock-world";

const scan = (bssid: string, rssi: number, channel = 6) => ({ bssid, ssid: "Home", channel, rssi });

/** A store whose clock we control. */
function store() {
  let t = 1_000;
  const s = new ScanStore(() => (t += 1000));
  const cycle = () => { for (let i = 0; i < 4; i++) s.pulse(true); };
  return { s, cycle };
}

describe("dedup visualisation (the device re-reports a network only when RSSI moves by 6 dB)", () => {
  it("highlights networks whose latest report moved by at least 6 dB", () => {
    const { s, cycle } = store();
    s.ingest(scan("aa", -50));
    s.ingest(scan("bb", -70));
    cycle();
    s.ingest(scan("aa", -58)); // moved 8 dB
    s.ingest(scan("bb", -64)); // moved 6 dB
    const rows = deltaView(s.networks(), s.cycle);
    expect(rows.map((r) => [r.record.bssid, r.delta])).toEqual([["aa", -8], ["bb", 6]]);
  });

  it("ignores networks reported once, and ones that moved less than the threshold", () => {
    const { s, cycle } = store();
    s.ingest(scan("aa", -50));
    cycle();
    s.ingest(scan("bb", -60));
    s.ingest(scan("aa", -53)); // 3 dB: not something the device would send, and not highlighted either
    expect(deltaView(s.networks(), s.cycle)).toEqual([]);
    expect(lastDelta(s.get("bb")!)).toBeNull();
  });

  it("drops a network from the view once it has been quiet for a cycle", () => {
    const { s, cycle } = store();
    s.ingest(scan("aa", -50));
    cycle();
    s.ingest(scan("aa", -70));
    expect(deltaView(s.networks(), s.cycle)).toHaveLength(1);
    cycle();
    cycle();
    expect(deltaView(s.networks(), s.cycle)).toEqual([]);
  });

  it("keeps the RSSI history of each network", () => {
    const { s, cycle } = store();
    s.ingest(scan("aa", -50));
    cycle();
    s.ingest(scan("aa", -60));
    expect(s.get("aa")!.history.map((p) => [p.cycle, p.rssi])).toEqual([[0, -50], [1, -60]]);
    expect(s.get("aa")!.count).toBe(2);
  });

  it("the simulated device obeys the same rule: every re-report moved by 6 dB or more", () => {
    const world = new MockWorld(11);
    const last = new Map<string, number>();
    let reports = 0;
    let channel = 1;
    for (let i = 0; i < 800; i++) {
      const step = world.step(true, channel, 0);
      channel = step.channel;
      for (const m of step.telemetry) {
        if (m.type !== "scan") continue;
        reports++;
        const prev = last.get(m.bssid);
        if (prev !== undefined) expect(Math.abs(m.rssi - prev)).toBeGreaterThanOrEqual(RSSI_DELTA_DB);
        last.set(m.bssid, m.rssi);
      }
    }
    expect(reports).toBeGreaterThan(20);
    expect(last.size).toBe(9);
  });
});

describe("snapshots hold what one scan heard", () => {
  it("a network that stopped being heard shows up as disappeared between two scans", async () => {
    const { createMonitorKernel, diffSnapshots } = await import("../../../src/shared/monitor");
    let t = 0;
    const k = createMonitorKernel(() => (t += 100), { getItem: () => null, setItem: () => {} });
    const status = (scanning: boolean) => k.ingest({ type: "status", activeChannel: 1, freeHeap: 1, cpuLoad: 1, dropped: 0, scanning });
    const net = (bssid: string, channel = 6, rssi = -50) => k.ingest({ type: "scan", bssid, channel, rssi, ssid: bssid });

    status(true); // scan 1 hears A and B
    net("A"); net("B");
    status(false); // stop: saved automatically

    status(true); // scan 2 hears A (moved channel) and C
    net("A", 11, -70); net("C");
    status(false);

    const [one, two] = k.scans.snapshots();
    expect(one.networks.map((n) => n.bssid)).toEqual(["A", "B"]);
    expect(two.networks.map((n) => n.bssid)).toEqual(["A", "C"]);
    const diff = diffSnapshots(one, two);
    expect(diff.added.map((n) => n.bssid)).toEqual(["C"]);
    expect(diff.removed.map((n) => n.bssid)).toEqual(["B"]);
    expect(diff.channelChanged.map((c) => c.after.bssid)).toEqual(["A"]);
    expect(diff.rssiChanged.map((c) => c.delta)).toEqual([-20]);
  });

  it("attaches WPS metadata to existing network records and emits changes", () => {
    const { s } = store();
    let changeFired = false;
    s.onChange(() => {
      changeFired = true;
    });
    s.ingest(scan("aa:bb:cc:dd:ee:ff", -50));
    expect(s.get("aa:bb:cc:dd:ee:ff")?.wps).toBeUndefined();

    s.ingestWps({
      bssid: "aa:bb:cc:dd:ee:ff",
      version: 0x20,
      state: 2,
      locked: true,
      selectedRegistrar: false,
      deviceName: "Archer AX50",
      manufacturer: "TP-Link",
      modelName: "AX3000",
    });

    const net = s.get("aa:bb:cc:dd:ee:ff");
    expect(net?.wps).toEqual({
      version: 0x20,
      state: 2,
      locked: true,
      selectedRegistrar: false,
      deviceName: "Archer AX50",
      manufacturer: "TP-Link",
      modelName: "AX3000",
    });
    expect(changeFired).toBe(true);
  });
});
