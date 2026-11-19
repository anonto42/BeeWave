import { describe, expect, it } from "vitest";
import { GpsTracker } from "../../../src/shared/gps";
import {
  channelToFrequencyMhz,
  formatWigleTimestamp,
  generateWigleCsv,
  networkToWigleAuthMode,
  type NetworkRecord,
} from "../../../src/shared/monitor";

describe("WiGLE Wardriving Export", () => {
  it("converts channels to 2.4 GHz frequencies", () => {
    expect(channelToFrequencyMhz(1)).toBe(2412);
    expect(channelToFrequencyMhz(6)).toBe(2437);
    expect(channelToFrequencyMhz(11)).toBe(2462);
    expect(channelToFrequencyMhz(13)).toBe(2472);
    expect(channelToFrequencyMhz(14)).toBe(2484);
  });

  it("maps security protocol and PMF to standard WiGLE capability strings", () => {
    expect(networkToWigleAuthMode(undefined)).toBe("[ESS]");
    expect(networkToWigleAuthMode({ protocol: "open", pmf: "none" })).toBe("[OPEN][ESS]");
    expect(networkToWigleAuthMode({ protocol: "wep", pmf: "none" })).toBe("[WEP][ESS]");
    expect(networkToWigleAuthMode({ protocol: "wpa", pmf: "none" })).toBe("[WPA-PSK-TKIP][ESS]");
    expect(networkToWigleAuthMode({ protocol: "wpa2", pmf: "none" })).toBe("[WPA2-PSK-CCMP][RSN][ESS]");
    expect(networkToWigleAuthMode({ protocol: "wpa2", pmf: "capable" })).toBe("[WPA2-PSK-CCMP][RSN][ESS][PMF-C]");
    expect(networkToWigleAuthMode({ protocol: "wpa3", pmf: "required" })).toBe("[WPA3-SAE-CCMP][RSN][ESS][PMF-R]");
    expect(networkToWigleAuthMode({ protocol: "wpa2Wpa3", pmf: "capable" })).toBe("[WPA2-PSK-CCMP][WPA3-SAE-CCMP][RSN][ESS][PMF-C]");
    expect(networkToWigleAuthMode({ protocol: "owe", pmf: "required" })).toBe("[OWE-CCMP][RSN][ESS][PMF-R]");
    expect(networkToWigleAuthMode({ protocol: "enterprise", pmf: "required" })).toBe("[WPA2-EAP-CCMP][RSN][ESS][PMF-R]");
  });

  it("formats timestamps to WiGLE UTC specification", () => {
    // 2026-11-19 12:30:45 UTC
    const ts = Date.UTC(2026, 10, 19, 12, 30, 45);
    expect(formatWigleTimestamp(ts)).toBe("2026-11-19 12:30:45");
  });

  it("generates valid WiGLE CSV with header lines and network records", () => {
    const net1: NetworkRecord = {
      bssid: "00:11:22:33:44:55",
      ssid: "CoffeeShop",
      channel: 6,
      rssi: -65,
      firstSeen: Date.UTC(2026, 10, 19, 10, 0, 0),
      lastSeen: Date.UTC(2026, 10, 19, 10, 5, 0),
      count: 5,
      history: [],
      security: { protocol: "wpa2", pmf: "none" },
      gps: { lat: 37.774929, lon: -122.419416, alt: 15.2, accuracy: 4.5, at: 1000 },
    };

    const net2: NetworkRecord = {
      bssid: "aa:bb:cc:dd:ee:ff",
      ssid: 'Cafe, "Special"',
      channel: 1,
      rssi: -72,
      firstSeen: Date.UTC(2026, 10, 19, 10, 1, 0),
      lastSeen: Date.UTC(2026, 10, 19, 10, 6, 0),
      count: 2,
      history: [],
      security: { protocol: "open", pmf: "none" },
    };

    const csv = generateWigleCsv([net1, net2]);
    const lines = csv.split("\r\n").filter(Boolean);

    expect(lines[0]).toContain("WigleWifi-1.4");
    expect(lines[1]).toBe("MAC,SSID,AuthMode,FirstSeen,Channel,Frequency,RSSI,CurrentLatitude,CurrentLongitude,AltitudeMeters,AccuracyMeters,Type");

    // Check row 1
    expect(lines[2]).toBe(
      "00:11:22:33:44:55,CoffeeShop,[WPA2-PSK-CCMP][RSN][ESS],2026-11-19 10:00:00,6,2437,-65,37.774929,-122.419416,15.2,4.5,WIFI"
    );

    // Check row 2 (escaped commas/quotes, fallback GPS 0.0)
    expect(lines[3]).toBe(
      'aa:bb:cc:dd:ee:ff,"Cafe, ""Special""", [OPEN][ESS],2026-11-19 10:01:00,1,2412,-72,0.000000,0.000000,0.0,0.0,WIFI'.replace(", [OPEN]", ",[OPEN]")
    );
  });
});

describe("GpsTracker", () => {
  it("tracks coordinates and notifies listeners", () => {
    let watchCallback: ((pos: GeolocationPosition) => void) | null = null;
    let clearCalled = false;

    const fakeGeo: Partial<Geolocation> = {
      watchPosition: (cb) => {
        watchCallback = cb;
        return 101;
      },
      clearWatch: (id) => {
        if (id === 101) clearCalled = true;
      },
    };

    const tracker = new GpsTracker(fakeGeo as Geolocation);
    expect(tracker.isSupported).toBe(true);
    expect(tracker.isActive).toBe(false);
    expect(tracker.coords).toBeNull();

    let changes = 0;
    tracker.onChange(() => {
      changes++;
    });

    tracker.start();
    expect(tracker.isActive).toBe(true);

    // Simulate location update
    watchCallback!({
      coords: {
        latitude: 40.7128,
        longitude: -74.006,
        altitude: 10,
        accuracy: 3.5,
        altitudeAccuracy: null,
        heading: null,
        speed: null,
      },
      timestamp: 1700000000000,
    } as unknown as GeolocationPosition);

    expect(tracker.coords).toEqual({
      lat: 40.7128,
      lon: -74.006,
      alt: 10,
      accuracy: 3.5,
      at: 1700000000000,
    });
    expect(changes).toBeGreaterThan(0);

    tracker.stop();
    expect(tracker.isActive).toBe(false);
    expect(clearCalled).toBe(true);
  });
});
