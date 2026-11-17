import { describe, expect, it } from "vitest";
import {
  DEFAULT_DEVICE_SETTINGS, DEFAULT_PHONE_SETTINGS, buildDeviceExport, buildPhoneExport, clonePhone, parseDeviceImport, parsePhoneImport,
} from "../../../src/shared/settings";

const pin = { bssid: "aa:bb:cc:dd:ee:ff", ssid: "Home", label: "front door", appear: true, disappear: true, deltaDb: 6, enabled: true };

describe("phone settings export and import", () => {
  it("round-trips settings and the watchlist, with an explicit schema version", () => {
    const s = clonePhone(DEFAULT_PHONE_SETTINGS);
    s.ui.theme = "dark";
    const file = buildPhoneExport(s, [pin]);
    expect(file).toMatchObject({ schema: 1, kind: "beewave-phone-settings" });
    const r = parsePhoneImport(JSON.stringify(file), DEFAULT_PHONE_SETTINGS, []);
    expect(r.ok && r.settings.ui.theme).toBe("dark");
    expect(r.ok && r.watchlist).toEqual([pin]);
    expect(r.ok && r.changes.map((c) => c.path)).toEqual(["ui.theme"]);
    expect(r.ok && r.watchlistChanged).toBe(true);
  });

  it("includes the log and scan history only when asked", () => {
    expect(buildPhoneExport(DEFAULT_PHONE_SETTINGS, [])).not.toHaveProperty("sessionLog");
    expect(buildPhoneExport(DEFAULT_PHONE_SETTINGS, [], { sessionLog: [{ at: 1, text: "x" }] }).sessionLog).toHaveLength(1);
  });

  it("refuses unknown keys, wrong schema and wrong kind, and reports every problem", () => {
    const base = buildPhoneExport(DEFAULT_PHONE_SETTINGS, []);
    const attempt = (patch: object) => parsePhoneImport(JSON.stringify({ ...base, ...patch }), DEFAULT_PHONE_SETTINGS, []);
    expect(attempt({ surprise: 1 })).toMatchObject({ ok: false, errors: expect.arrayContaining(["unknown key surprise"]) });
    expect(attempt({ schema: 2 })).toMatchObject({ ok: false, errors: [expect.stringContaining("unsupported schema version 2")] });
    expect(attempt({ kind: "other" })).toMatchObject({ ok: false });
    expect(attempt({ settings: { ui: { theme: "neon" } } })).toMatchObject({ ok: false });
    expect(parsePhoneImport("nope", DEFAULT_PHONE_SETTINGS, [])).toEqual({ ok: false, errors: ["not valid JSON"] });
  });

  it("validates watchlist entries", () => {
    const base = buildPhoneExport(DEFAULT_PHONE_SETTINGS, []);
    const bad = (e: object) => parsePhoneImport(JSON.stringify({ ...base, watchlist: [e] }), DEFAULT_PHONE_SETTINGS, []).ok;
    expect(bad({ bssid: "not-a-bssid" })).toBe(false);
    expect(bad({ bssid: pin.bssid, deltaDb: 99 })).toBe(false);
    expect(bad({ bssid: pin.bssid, mystery: 1 })).toBe(false);
    expect(bad({ bssid: pin.bssid })).toBe(true); // everything else defaults
  });

  it("reports nothing to change for an identical file", () => {
    const r = parsePhoneImport(JSON.stringify(buildPhoneExport(DEFAULT_PHONE_SETTINGS, [])), DEFAULT_PHONE_SETTINGS, []);
    expect(r.ok && r.changes).toEqual([]);
    expect(r.ok && r.watchlistChanged).toBe(false);
  });
});

describe("device settings export and import", () => {
  it("round-trips, so a fleet can be provisioned from one file", () => {
    const file = buildDeviceExport(DEFAULT_DEVICE_SETTINGS);
    expect(file).toMatchObject({ schema: 1, kind: "beewave-device-settings" });
    const r = parseDeviceImport(JSON.stringify(file));
    expect(r).toEqual({ ok: true, settings: DEFAULT_DEVICE_SETTINGS });
  });

  it("rejects unknown and missing keys, out-of-range values and other files", () => {
    const file = JSON.parse(JSON.stringify(buildDeviceExport(DEFAULT_DEVICE_SETTINGS)));
    const attempt = (mutate: (f: typeof file) => void) => {
      const f = JSON.parse(JSON.stringify(file));
      mutate(f);
      return parseDeviceImport(JSON.stringify(f));
    };
    expect(attempt((f) => void (f.device.scan.sparkle = 1)).ok).toBe(false);
    expect(attempt((f) => void delete f.device.alerts.deauthWindowMs).ok).toBe(false);
    expect(attempt((f) => void (f.device.scan.dedupRssiDb = 99)).ok).toBe(false);
    expect(attempt((f) => void (f.kind = "beewave-phone-settings")).ok).toBe(false);
    expect(attempt((f) => void (f.device.extra = {})).ok).toBe(false);
    expect(attempt((f) => void (f.schema = 7)).ok).toBe(false);
  });
});
