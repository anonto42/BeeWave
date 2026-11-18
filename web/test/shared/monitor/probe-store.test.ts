import { describe, expect, it } from "vitest";
import { ProbeStore } from "../../../src/shared/monitor";

describe("ProbeStore", () => {
  it("ingests probe observations and aggregates probed SSIDs per client", () => {
    let t = 1000;
    const store = new ProbeStore(() => t);

    store.ingest({
      type: "probeResult",
      client: "da:a1:19:22:33:44",
      channel: 1,
      rssi: -65,
      ssid: "HomeWiFi",
    });

    t = 2000;
    store.ingest({
      type: "probeResult",
      client: "da:a1:19:22:33:44",
      channel: 6,
      rssi: -60,
      ssid: "OfficeGuest",
    });

    const list = store.list();
    expect(list).toHaveLength(1);
    const client = list[0];
    expect(client.client).toBe("da:a1:19:22:33:44");
    expect(client.channel).toBe(6);
    expect(client.rssi).toBe(-60);
    expect(client.probedSsids).toEqual(["HomeWiFi", "OfficeGuest"]);
    expect(client.probeCount).toBe(2);
    expect(client.firstSeen).toBe(1000);
    expect(client.lastSeen).toBe(2000);
  });

  it("handles wildcard probes with empty string", () => {
    const store = new ProbeStore();
    store.ingest({
      type: "probeResult",
      client: "f0:18:98:55:66:77",
      channel: 11,
      rssi: -70,
      ssid: "",
    });
    const list = store.list();
    expect(list).toHaveLength(1);
    expect(list[0].probedSsids).toEqual([]);
  });

  it("clears entries when clear() is called", () => {
    const store = new ProbeStore();
    store.ingest({
      type: "probeResult",
      client: "da:a1:19:22:33:44",
      channel: 1,
      rssi: -65,
      ssid: "HomeWiFi",
    });
    expect(store.list()).toHaveLength(1);
    store.clear();
    expect(store.list()).toHaveLength(0);
  });
});
