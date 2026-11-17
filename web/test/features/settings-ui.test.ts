import { describe, expect, it } from "vitest";
import { barHtml, bannerText } from "../../src/features/settings/template";
import { KeyStore } from "../../src/shared/auth";
import { WatchlistEngine, fullEntry, type WatchEntry } from "../../src/shared/monitor";
import { DEFAULT_DEVICE_SETTINGS, cloneSettings, withField } from "../../src/shared/settings";
import { MockSettings } from "../../src/shared/transport/mock-settings";
import { MockWorld } from "../../src/shared/transport/mock-world";

const save = (m: MockSettings, settings = m.settings) => m.handle({ type: "saveSettingsAll", settings }, { scanning: false })!.replies[0];

describe("the simulated device's settings (same rules as the core)", () => {
  it("persists a valid save and counts the revision", () => {
    const m = new MockSettings();
    const next = withField(cloneSettings(DEFAULT_DEVICE_SETTINGS), "scan.dedupRssiDb", 12);
    expect(save(m, next)).toEqual({ type: "settingsSaved", revision: 1, ok: true, reason: null });
    expect(save(m, next)).toMatchObject({ revision: 2, ok: true });
    const all = m.handle({ type: "getSettingsAll" }, { scanning: false })!.replies[0];
    expect(all).toMatchObject({ type: "settingsAll", revision: 2, settings: { scan: { dedupRssiDb: 12 } } });
  });

  it("a refusal names the reason and writes nothing", () => {
    const m = new MockSettings();
    const bad = withField(cloneSettings(DEFAULT_DEVICE_SETTINGS), "scan.dedupRssiDb", 99);
    expect(save(m, bad)).toEqual({ type: "settingsSaved", revision: 0, ok: false, reason: "dedupRssiRange" });
    expect(m.revision).toBe(0);
    expect(m.settings.scan.dedupRssiDb).toBe(6);
    const pair = withField(withField(cloneSettings(DEFAULT_DEVICE_SETTINGS), "scan.scanIntervalMs", 900), "alerts.deauthWindowMs", 1000);
    expect(save(m, pair)).toMatchObject({ ok: false, reason: "windowTooShort" });
  });

  it("factory reset restores defaults and the revision, and the reply is a reboot (no telemetry)", () => {
    const m = new MockSettings();
    save(m, withField(cloneSettings(DEFAULT_DEVICE_SETTINGS), "scan.dedupRssiDb", 12));
    const r = m.handle({ type: "factoryReset" }, { scanning: false })!;
    expect(r).toEqual({ replies: [], factoryReset: true });
    expect([m.revision, m.settings.scan.dedupRssiDb]).toEqual([0, 6]);
  });

  it("the self-test is refused while scanning", () => {
    const m = new MockSettings();
    expect(m.handle({ type: "runSelfTest" }, { scanning: true })!.replies[0]).toMatchObject({ busy: true, radio: false });
    expect(m.handle({ type: "runSelfTest" }, { scanning: false })!.replies[0]).toMatchObject({ busy: false, radio: true, nvs: true, ble: true, otaSlot: true });
  });

  it("storage info carries the revision; device info says nothing is enabled", () => {
    const m = new MockSettings();
    save(m, withField(cloneSettings(DEFAULT_DEVICE_SETTINGS), "scan.dedupRssiDb", 12));
    expect(m.handle({ type: "getStorageInfo" }, { scanning: false })!.replies[0]).toMatchObject({ settingsRevision: 1, ota0Active: true });
    expect(m.handle({ type: "getDeviceInfo" }, { scanning: false })!.replies[0]).toMatchObject({ requestConnParams: false, bonding: false, coexDebug: false });
  });

  it("ignores commands it does not own", () => {
    expect(new MockSettings().handle({ type: "reboot" }, { scanning: false })).toBeNull();
  });

  it("saved scan settings change what the simulated sniffer reports", () => {
    const run = (apply: (w: MockWorld) => void) => {
      const w = new MockWorld(2);
      apply(w);
      let ch = 1;
      const seen = new Set<string>();
      for (let i = 0; i < 300; i++) {
        const s = w.step(true, ch, 0);
        ch = s.channel;
        for (const m of s.telemetry) if (m.type === "scan") seen.add(m.ssid);
      }
      return seen;
    };
    expect(run(() => {}).size).toBeGreaterThan(3);
    expect([...run((w) => w.applyScanSettings({ dedupRssiDb: 6, ssidFilter: "office", channelFilterMask: 0 }))]).toEqual(["Office"]);
    const onlyCh1 = run((w) => w.applyScanSettings({ dedupRssiDb: 6, ssidFilter: "", channelFilterMask: 1 }));
    expect([...onlyCh1].sort()).toEqual(["BeeWave-lab", "Office"]);
  });
});

describe("watchlist rules per entry", () => {
  const BSSID = "aa:bb:cc:dd:ee:ff";
  const engine = (entry: WatchEntry) => new WatchlistEngine((b) => (b === BSSID ? entry : undefined), () => [entry]);
  const scan = (rssi: number) => ({ bssid: BSSID, ssid: "Home", channel: 6, rssi });
  const base: WatchEntry = { bssid: BSSID, ssid: "Home" };

  it("fills in the defaults for older saved entries", () => {
    expect(fullEntry(base)).toEqual({ ...base, label: "", appear: true, disappear: true, deltaDb: 6, enabled: true });
  });

  it("respects the per-rule switches, the dB threshold and the on/off switch", () => {
    const quiet = engine({ ...base, appear: false, deltaDb: 0 });
    expect(quiet.observe(scan(-50), 0)).toEqual([]); // appearing is off
    expect(quiet.observe(scan(-90), 1)).toEqual([]); // a move alert is off (0)
    expect(quiet.sweep(10)[0]?.title).toBe("Watched network disappeared");

    const noGone = engine({ ...base, disappear: false });
    noGone.observe(scan(-50), 0);
    expect(noGone.sweep(10)).toEqual([]);

    const wide = engine({ ...base, deltaDb: 15 });
    wide.observe(scan(-50), 0);
    expect(wide.observe(scan(-60), 1)).toEqual([]);
    expect(wide.observe(scan(-76), 2)[0]?.title).toBe("Watched network moved");

    const off = engine({ ...base, enabled: false });
    expect(off.observe(scan(-50), 0)).toEqual([]);
    expect(off.sweep(10)).toEqual([]);
  });

  it("uses the label in alert text when there is one", () => {
    const e = engine({ ...base, label: "front door" });
    expect(e.observe(scan(-50), 0)[0].detail).toContain("front door");
  });
});

describe("key persistence setting", () => {
  const mem = () => {
    const d = new Map<string, string>();
    return { getItem: (k: string) => d.get(k) ?? null, setItem: (k: string, v: string) => void d.set(k, v), removeItem: (k: string) => void d.delete(k), d };
  };
  const KEY = "ab".repeat(32);

  it("with remember-key off the key lives in memory only and any saved copy is removed", () => {
    const m = mem();
    const keys = new KeyStore(m);
    keys.set(KEY);
    expect(m.d.has("beewave.key")).toBe(true);
    keys.setPersist(false);
    expect(m.d.has("beewave.key")).toBe(false);
    expect(keys.get()).not.toBeNull();
    keys.set("cd".repeat(32));
    expect(m.d.has("beewave.key")).toBe(false);
  });

  it("turning remember-key back on saves the key it holds", () => {
    const m = mem();
    const keys = new KeyStore(m);
    keys.setPersist(false);
    keys.set(KEY);
    keys.setPersist(true);
    expect(m.d.get("beewave.key")).toBe(KEY);
  });
});

describe("settings banner and apply bar", () => {
  it("explains why device settings are unavailable", () => {
    expect(bannerText("ok", true, true)).toBeNull();
    expect(bannerText("unknown", false, false)).toMatch(/after the key handshake/);
    expect(bannerText("unknown", true, false)).toMatch(/Reading/);
    expect(bannerText("older", true, true)).toMatch(/older than the app/);
    expect(bannerText("newer", true, true)).toMatch(/newer than the app/);
    expect(bannerText("unsupported", true, false)).toMatch(/does not support/);
  });

  it("shows the unsaved count, the spinner and the refusal reason", () => {
    const bar = (o: Partial<Parameters<typeof barHtml>[0]> = {}) =>
      barHtml({ changes: 0, saving: false, ready: true, state: "idle", reason: null, revision: 4, ...o });
    expect(bar({ changes: 3 })).toContain("3 unsaved changes");
    expect(bar({ changes: 1 })).toContain("1 unsaved change<");
    expect(bar({ changes: 2, saving: true })).toContain("spinner");
    expect(bar({ changes: 2, state: "rejected", reason: "windowTooShort" })).toContain("at least twice the scan interval");
    expect(bar({ changes: 0 })).toContain("saved 4 times");
    // apply is disabled with nothing to send or while saving
    expect(bar()).toMatch(/data-apply disabled/);
    expect(bar({ changes: 1 })).not.toMatch(/data-apply disabled/);
    expect(bar({ changes: 1, saving: true })).toMatch(/data-apply disabled/);
    expect(bar({ changes: 1, ready: false })).toMatch(/data-apply disabled/);
  });
});
