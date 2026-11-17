import { describe, expect, it } from "vitest";
import { DEFAULT_DEVICE_SETTINGS, cloneSettings, validateDeviceSettings, withField, type FieldPath } from "../../../src/shared/settings";

/** Mirrors crates/core/src/features/settings/tests.rs: the same boundaries must give the same answers. */
const with_ = (changes: Array<[FieldPath, unknown]>) => changes.reduce((s, [p, v]) => withField(s, p, v), cloneSettings(DEFAULT_DEVICE_SETTINGS));
const reason = (changes: Array<[FieldPath, unknown]>) => validateDeviceSettings(with_(changes))?.reason ?? null;

describe("device settings validation (parity with the core)", () => {
  it("the defaults are valid", () => {
    expect(validateDeviceSettings(DEFAULT_DEVICE_SETTINGS)).toBeNull();
  });

  it("scan interval is 100 to 10000 (the window rule bites at the top)", () => {
    const at = (v: number) => reason([["scan.scanIntervalMs", v], ["alerts.deauthWindowMs", 5000]]);
    expect([at(99), at(100), at(2500), at(10_000), at(10_001), at(0)]).toEqual([
      "scanIntervalRange", null, null, "windowTooShort", "scanIntervalRange", "scanIntervalRange",
    ]);
  });

  it("dedup is 1 to 30 dB", () => {
    const at = (v: number) => reason([["scan.dedupRssiDb", v]]);
    expect([at(0), at(1), at(30), at(31), at(255)]).toEqual(["dedupRssiRange", null, null, "dedupRssiRange", "dedupRssiRange"]);
  });

  it("the channel filter may only use bits for channels 1 to 13", () => {
    const at = (v: number) => reason([["scan.channelFilterMask", v]]);
    expect([at(0), at(0b1_1111_1111_1111), at(1), at(1 << 13), at(65_535)]).toEqual([null, null, null, "channelMaskRange", "channelMaskRange"]);
  });

  it("the default channel is AUTO or 1 to 13", () => {
    const at = (v: number) => reason([["radio.defaultChannel", v]]);
    expect([at(0), at(1), at(13), at(14), at(255)]).toEqual([null, null, null, "defaultChannelRange", "defaultChannelRange"]);
  });

  it("hop dwell is 100 to 2000 ms", () => {
    const at = (v: number) => reason([["radio.hopDwellMs", v]]);
    expect([at(99), at(100), at(2000), at(2001), at(0)]).toEqual(["hopDwellRange", null, null, "hopDwellRange", "hopDwellRange"]);
  });

  it("status period is 250 to 5000 ms", () => {
    const at = (v: number) => reason([["telemetry.statusPeriodMs", v]]);
    expect([at(249), at(250), at(5000), at(5001), at(0)]).toEqual(["statusPeriodRange", null, null, "statusPeriodRange", "statusPeriodRange"]);
  });

  it("deauth threshold is 1 to 100", () => {
    const at = (v: number) => reason([["alerts.deauthThreshold", v]]);
    expect([at(0), at(1), at(100), at(101), at(255)]).toEqual(["deauthThresholdRange", null, null, "deauthThresholdRange", "deauthThresholdRange"]);
  });

  it("deauth window is 200 to 5000 ms", () => {
    const at = (v: number) => reason([["alerts.deauthWindowMs", v], ["scan.scanIntervalMs", 100]]);
    expect([at(199), at(200), at(5000), at(5001), at(0)]).toEqual(["deauthWindowRange", null, null, "deauthWindowRange", "deauthWindowRange"]);
  });

  it("a custom hop order needs 1-13 once each, at least one channel", () => {
    const custom = (list: number[]) => reason([["radio.hopOrder", "custom"], ["radio.customHopList", list]]);
    const pad = (l: number[]) => [...l, ...Array(13 - l.length).fill(0)];
    expect(custom(pad([]))).toBe("hopListEmpty");
    expect(custom(pad([6]))).toBeNull();
    expect(custom(pad([13, 1, 6, 11]))).toBeNull();
    expect(custom(pad([14]))).toBe("hopListInvalid");
    expect(custom(pad([255]))).toBe("hopListInvalid");
    expect(custom(pad([1, 6, 6]))).toBe("hopListInvalid");
    // ignored unless the order is custom
    expect(reason([["radio.hopOrder", "ascending1_6_11"]])).toBeNull();
  });

  it("the deauth window must span two scan intervals", () => {
    const at = (interval: number, window: number) => reason([["scan.scanIntervalMs", interval], ["alerts.deauthWindowMs", window]]);
    expect([at(500, 1000), at(501, 1000), at(2500, 5000), at(2501, 5000), at(100, 200), at(101, 200)]).toEqual([
      null, "windowTooShort", null, "windowTooShort", null, "windowTooShort",
    ]);
  });

  it("names the field that is wrong, so the UI can highlight it", () => {
    expect(validateDeviceSettings(with_([["scan.dedupRssiDb", 99]]))).toEqual({ field: "scan.dedupRssiDb", reason: "dedupRssiRange" });
    expect(validateDeviceSettings(with_([["alerts.deauthWindowMs", 100]]))?.field).toBe("alerts.deauthWindowMs");
  });

  it("an SSID filter over 32 bytes is caught on the client (multi-byte text counts in bytes)", () => {
    expect(reason([["scan.ssidFilter", "a".repeat(32)]])).toBeNull();
    expect(reason([["scan.ssidFilter", "é".repeat(17)]])).toBe("ssidTooLong");
  });
});
