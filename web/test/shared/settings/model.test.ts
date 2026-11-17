import { describe, expect, it } from "vitest";
import { DEFAULT_DEVICE_SETTINGS, DeviceSettingsModel, cloneSettings } from "../../../src/shared/settings";

const loaded = (revision = 3) => {
  const m = new DeviceSettingsModel();
  m.ingest({ type: "settingsAll", schema: 1, revision, settings: cloneSettings(DEFAULT_DEVICE_SETTINGS) });
  return m;
};

describe("DeviceSettingsModel", () => {
  it("is empty until the device has sent its settings", () => {
    const m = new DeviceSettingsModel();
    expect([m.loaded, m.dirtyCount(), m.support]).toEqual([false, 0, "unknown"]);
    m.set("scan.dedupRssiDb", 9); // nothing to edit yet
    expect(m.draft).toBeNull();
    expect(m.beginSave()).toBeNull();
  });

  it("tracks which fields and sections are dirty", () => {
    const m = loaded();
    m.set("scan.dedupRssiDb", 9);
    m.set("scan.ssidFilter", "cafe");
    m.set("alerts.deauthThreshold", 20);
    expect(m.dirty().sort()).toEqual(["alerts.deauthThreshold", "scan.dedupRssiDb", "scan.ssidFilter"]);
    expect([m.dirtyCount(), m.sectionDirty("scan"), m.sectionDirty("alerts"), m.sectionDirty("radio")]).toEqual([3, 2, 1, 0]);
    m.set("scan.dedupRssiDb", 6); // put it back: no longer dirty
    expect(m.dirtyCount()).toBe(2);
  });

  it("compares list fields by value", () => {
    const m = loaded();
    m.set("radio.customHopList", Array(13).fill(0));
    expect(m.dirtyCount()).toBe(0);
    m.set("radio.customHopList", [1, ...Array(12).fill(0)]);
    expect(m.dirtyCount()).toBe(1);
  });

  it("revert restores what the device confirmed", () => {
    const m = loaded();
    m.set("scan.dedupRssiDb", 9);
    m.revert();
    expect([m.dirtyCount(), m.draft!.scan.dedupRssiDb]).toEqual([0, 6]);
  });

  it("applying sends the whole draft once and shows saving until the device answers", () => {
    const m = loaded();
    m.set("scan.dedupRssiDb", 9);
    const r = m.beginSave();
    expect(r && "send" in r && r.send.scan.dedupRssiDb).toBe(9);
    expect(m.save.state).toBe("saving");
    expect(m.beginSave()).toBeNull(); // no double send while waiting
  });

  it("a confirmed save becomes the new baseline and updates the revision", () => {
    const m = loaded(3);
    m.set("scan.dedupRssiDb", 9);
    m.beginSave();
    m.ingest({ type: "settingsSaved", revision: 4, ok: true, reason: null });
    expect([m.save.state, m.revision, m.dirtyCount(), m.confirmed!.scan.dedupRssiDb]).toEqual(["idle", 4, 0, 9]);
  });

  it("a refused save keeps the confirmed values, names the reason and highlights the field", () => {
    const m = loaded(3);
    m.set("alerts.deauthWindowMs", 1000);
    m.set("scan.scanIntervalMs", 600); // 2 x 600 > 1000: the core refuses this pair
    // the client-side check agrees and does not even send
    const local = m.beginSave();
    expect(local).toMatchObject({ violation: { field: "alerts.deauthWindowMs", reason: "windowTooShort" } });

    const m2 = loaded(3);
    m2.set("scan.dedupRssiDb", 9);
    m2.beginSave();
    m2.ingest({ type: "settingsSaved", revision: 3, ok: false, reason: "dedupRssiRange" });
    expect(m2.save).toEqual({ state: "rejected", reason: "dedupRssiRange", field: "scan.dedupRssiDb" });
    expect(m2.confirmed!.scan.dedupRssiDb).toBe(6); // not considered changed
    expect(m2.revision).toBe(3);
    m2.set("scan.dedupRssiDb", 7); // editing the highlighted field clears the error
    expect(m2.save.state).toBe("idle");
  });

  it("a storage refusal is not about one field", () => {
    const m = loaded();
    m.set("scan.dedupRssiDb", 9);
    m.beginSave();
    m.ingest({ type: "settingsSaved", revision: 3, ok: false, reason: "storage" });
    expect(m.save).toEqual({ state: "rejected", reason: "storage", field: null });
  });

  it("detects a firmware with an older or newer settings layout", () => {
    const m = new DeviceSettingsModel();
    m.ingest({ type: "settingsAll", schema: 0, revision: 0, settings: cloneSettings(DEFAULT_DEVICE_SETTINGS) });
    expect(m.support).toBe("older");
    m.ingest({ type: "settingsAll", schema: 2, revision: 0, settings: cloneSettings(DEFAULT_DEVICE_SETTINGS) });
    expect(m.support).toBe("newer");
  });

  it("marks firmware that never answers as unsupported, but only while still unknown", () => {
    const m = new DeviceSettingsModel();
    m.markUnsupported();
    expect(m.support).toBe("unsupported");
    const ok = loaded();
    ok.markUnsupported();
    expect(ok.support).toBe("ok");
  });

  it("a dropped link clears the stale device values", () => {
    const m = loaded();
    m.set("scan.dedupRssiDb", 9);
    m.reset();
    expect([m.loaded, m.dirtyCount(), m.revision, m.support]).toEqual([false, 0, 0, "unknown"]);
  });
});
