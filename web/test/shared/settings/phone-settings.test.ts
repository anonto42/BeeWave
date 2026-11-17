import { describe, expect, it } from "vitest";
import { DEFAULT_PHONE_SETTINGS, PhoneSettingsStore, clonePhone, diffPhoneSettings, parsePhoneSettings } from "../../../src/shared/settings";

const memory = (initial?: string) => {
  const data = new Map<string, string>(initial ? [["beewave.phone-settings", initial]] : []);
  return {
    getItem: (k: string) => data.get(k) ?? null,
    setItem: (k: string, v: string) => void data.set(k, v),
    removeItem: (k: string) => void data.delete(k),
    data,
  };
};

describe("phone settings parsing", () => {
  it("accepts the defaults and partial objects (missing keys take defaults)", () => {
    expect(parsePhoneSettings(DEFAULT_PHONE_SETTINGS)).toEqual({ ok: true, value: DEFAULT_PHONE_SETTINGS });
    const r = parsePhoneSettings({ ui: { theme: "dark" } });
    expect(r.ok && r.value.ui.theme).toBe("dark");
    expect(r.ok && r.value.ui.refreshHz).toBe(4);
  });

  it("rejects unknown keys at every level instead of ignoring them", () => {
    const r = parsePhoneSettings({ ui: { theme: "dark", sparkle: true }, extra: 1, session: { nope: 1 } });
    expect(r.ok).toBe(false);
    expect(!r.ok && r.errors).toEqual(expect.arrayContaining(["unknown key ui.sparkle", "unknown key extra", "unknown key session.nope"]));
  });

  it("checks types and ranges", () => {
    const bad = (s: object) => !parsePhoneSettings(s).ok;
    expect(bad({ ui: { theme: "neon" } })).toBe(true);
    expect(bad({ ui: { refreshHz: 0 } })).toBe(true);
    expect(bad({ ui: { refreshHz: 11 } })).toBe(true);
    expect(bad({ ui: { refreshHz: 2.5 } })).toBe(true);
    expect(bad({ ui: { refreshHz: 10 } })).toBe(false);
    expect(bad({ ui: { visibleColumns: [] } })).toBe(true);
    expect(bad({ ui: { visibleColumns: ["ssid", "ssid"] } })).toBe(true);
    expect(bad({ ui: { visibleColumns: ["ssid", "colour"] } })).toBe(true);
    expect(bad({ session: { autoReconnect: "yes" } })).toBe(true);
    expect(bad({ ota: { expectedVersion: "x".repeat(17) } })).toBe(true);
    expect(parsePhoneSettings("nope").ok).toBe(false);
    expect(parsePhoneSettings([]).ok).toBe(false);
  });

  it("lists what differs", () => {
    const b = clonePhone(DEFAULT_PHONE_SETTINGS);
    b.ui.theme = "light";
    b.ui.visibleColumns = ["ssid", "rssi"];
    expect(diffPhoneSettings(DEFAULT_PHONE_SETTINGS, b).map((c) => [c.path, c.to])).toEqual([
      ["ui.theme", "light"],
      ["ui.visibleColumns", ["ssid", "rssi"]],
    ]);
  });
});

describe("PhoneSettingsStore", () => {
  it("saves immediately and reloads", () => {
    const m = memory();
    const a = new PhoneSettingsStore(m);
    let seen = 0;
    a.onChange(() => seen++);
    a.update((s) => void (s.ui.theme = "dark"));
    expect(seen).toBe(1);
    expect(new PhoneSettingsStore(m).get().ui.theme).toBe("dark");
  });

  it("falls back to the defaults for corrupt or foreign stored data", () => {
    expect(new PhoneSettingsStore(memory("{not json")).get()).toEqual(DEFAULT_PHONE_SETTINGS);
    expect(new PhoneSettingsStore(memory(JSON.stringify({ ui: { theme: "neon" } }))).get()).toEqual(DEFAULT_PHONE_SETTINGS);
  });

  it("keeps working in memory when storage is blocked", () => {
    const broken = { getItem: () => { throw new Error("x"); }, setItem: () => { throw new Error("x"); }, removeItem: () => { throw new Error("x"); } };
    const s = new PhoneSettingsStore(broken);
    s.update((d) => void (d.ui.refreshHz = 9));
    expect(s.get().ui.refreshHz).toBe(9);
  });
});
