import { describe, expect, it } from "vitest";
import { chartSvg } from "../../src/features/channel-chart/chart";
import { visible } from "../../src/features/anomalies/filter";
import { DEFAULT_QUERY, filterAndSort, nextQuery } from "../../src/features/results/filter";
import type { AlertRecord, NetworkRecord } from "../../src/shared/monitor";

const rec = (over: Partial<NetworkRecord>): NetworkRecord => ({
  bssid: "aa:bb:cc:00:00:01",
  ssid: "Home",
  channel: 6,
  rssi: -50,
  firstSeen: 1000,
  lastSeen: 2000,
  count: 1,
  history: [],
  ...over,
});

const nets = [
  rec({ bssid: "3c:71:bf:00:00:01", ssid: "lab", channel: 1, rssi: -45, firstSeen: 100, lastSeen: 900, count: 9 }),
  rec({ bssid: "50:c7:bf:00:00:02", ssid: "Home", channel: 6, rssi: -70, firstSeen: 300, lastSeen: 500, count: 2 }),
  rec({ bssid: "02:00:00:00:00:03", ssid: "cafe", channel: 6, rssi: -60, firstSeen: 200, lastSeen: 700, count: 4 }),
  rec({ bssid: "fc:ec:da:00:00:04", ssid: "", channel: 11, rssi: -80, firstSeen: 400, lastSeen: 800, count: 1 }),
];

const order = (q: Partial<typeof DEFAULT_QUERY>) => filterAndSort(nets, { ...DEFAULT_QUERY, ...q }).map((r) => r.record.ssid || "(hidden)");

describe("results table", () => {
  it("sorts by any column in either direction", () => {
    expect(order({ sort: "rssi", dir: "desc" })).toEqual(["lab", "cafe", "Home", "(hidden)"]);
    expect(order({ sort: "ssid", dir: "asc" })).toEqual(["(hidden)", "cafe", "Home", "lab"]); // text ignores case
    expect(order({ sort: "channel", dir: "asc" }).slice(0, 1)).toEqual(["lab"]);
    expect(order({ sort: "firstSeen", dir: "asc" })).toEqual(["lab", "cafe", "Home", "(hidden)"]);
    expect(order({ sort: "lastSeen", dir: "desc" })).toEqual(["lab", "(hidden)", "cafe", "Home"]);
    expect(order({ sort: "count", dir: "desc" })).toEqual(["lab", "cafe", "Home", "(hidden)"]);
    expect(order({ sort: "vendor", dir: "asc" })).toEqual(["lab", "cafe", "Home", "(hidden)"]); // Espressif, Randomized, TP-Link, Ubiquiti
  });

  it("filters by SSID substring (any case) and by channel, together", () => {
    expect(order({ ssid: "OM" })).toEqual(["Home"]);
    expect(order({ channel: 6, sort: "rssi" })).toEqual(["cafe", "Home"]);
    expect(order({ ssid: "o", channel: 6 })).toEqual(["Home"]);
    expect(order({ ssid: "nope" })).toEqual([]);
  });

  it("clicking a column flips its direction; a new column starts sensibly", () => {
    const rssi = nextQuery(DEFAULT_QUERY, "rssi"); // was rssi desc
    expect(rssi.dir).toBe("asc");
    expect(nextQuery(DEFAULT_QUERY, "ssid")).toMatchObject({ sort: "ssid", dir: "asc" });
    expect(nextQuery(DEFAULT_QUERY, "count")).toMatchObject({ sort: "count", dir: "desc" });
  });

  it("shows the vendor, labelling random addresses", () => {
    const vendors = Object.fromEntries(filterAndSort(nets, DEFAULT_QUERY).map((r) => [r.record.ssid || "(hidden)", r.vendor]));
    expect(vendors).toMatchObject({ lab: "Espressif", Home: "TP-Link", cafe: "Randomized", "(hidden)": "Ubiquiti" });
  });
});

describe("alert feed filter", () => {
  const a = (id: number, source: AlertRecord["source"]): AlertRecord => ({ id, at: id, source, severity: "info", title: "", detail: "" });
  const list = [a(4, "ble"), a(3, "deauth"), a(2, "anomaly"), a(1, "deauth")];

  it("shows everything or one source, keeping the order", () => {
    expect(visible(list, "all").map((x) => x.id)).toEqual([4, 3, 2, 1]);
    expect(visible(list, "deauth").map((x) => x.id)).toEqual([3, 1]);
    expect(visible(list, "watchlist")).toEqual([]);
  });
});

describe("channel chart", () => {
  it("draws two bars and a label for each of the 13 channels", () => {
    const svg = chartSvg([2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0], [5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]);
    expect(svg.match(/class="bar-ap"/g)).toHaveLength(13);
    expect(svg.match(/class="bar-act"/g)).toHaveLength(13);
    expect(svg.match(/<text /g)).toHaveLength(13);
  });

  it("scales each series to its own maximum and never divides by zero", () => {
    const empty = chartSvg(Array(13).fill(0), Array(13).fill(0));
    expect(empty).not.toContain("NaN");
    const svg = chartSvg([4, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    const heights = [...svg.matchAll(/class="bar-ap"[^>]*height="([\d.]+)"/g)].map((m) => Number(m[1]));
    expect(heights[0]).toBeCloseTo(heights[1] * 2, 0);
    const act = [...svg.matchAll(/class="bar-act"[^>]*height="([\d.]+)"/g)].map((m) => Number(m[1]));
    expect(act[0]).toBeGreaterThan(0);
  });
});
