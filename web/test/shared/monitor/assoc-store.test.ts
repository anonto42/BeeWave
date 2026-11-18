import { describe, expect, it } from "vitest";
import { AssocStore } from "../../../src/shared/monitor";

describe("AssocStore", () => {
  it("ingests association observations and maps stations to BSSIDs", () => {
    let t = 1000;
    const store = new AssocStore(() => t);

    store.ingest({
      type: "assocResult",
      bssid: "00:11:22:33:44:55",
      client: "da:a1:19:22:33:44",
      channel: 1,
      rssi: -65,
    });

    t = 2000;
    store.ingest({
      type: "assocResult",
      bssid: "00:11:22:33:44:55",
      client: "da:a1:19:22:33:44",
      channel: 1,
      rssi: -58,
    });

    const list = store.list();
    expect(list).toHaveLength(1);
    const station = list[0];
    expect(station.bssid).toBe("00:11:22:33:44:55");
    expect(station.client).toBe("da:a1:19:22:33:44");
    expect(station.channel).toBe(1);
    expect(station.rssi).toBe(-58);
    expect(station.frameCount).toBe(2);
    expect(station.firstSeen).toBe(1000);
    expect(station.lastSeen).toBe(2000);

    const bssidStations = store.stationsFor("00:11:22:33:44:55");
    expect(bssidStations).toHaveLength(1);

    const clientAssoc = store.associationFor("da:a1:19:22:33:44");
    expect(clientAssoc?.bssid).toBe("00:11:22:33:44:55");
  });

  it("bounds maximum tracked associations", () => {
    const store = new AssocStore();
    for (let i = 0; i < 70; i++) {
      const hex = i.toString(16).padStart(2, "0");
      store.ingest({
        type: "assocResult",
        bssid: "00:11:22:33:44:55",
        client: `00:00:00:00:00:${hex}`,
        channel: 6,
        rssi: -70,
      });
    }

    expect(store.list().length).toBeLessThanOrEqual(64);
  });

  it("clears all associations on reset", () => {
    const store = new AssocStore();
    store.ingest({
      type: "assocResult",
      bssid: "00:11:22:33:44:55",
      client: "da:a1:19:22:33:44",
      channel: 1,
      rssi: -65,
    });
    expect(store.list()).toHaveLength(1);
    store.clear();
    expect(store.list()).toHaveLength(0);
  });
});
