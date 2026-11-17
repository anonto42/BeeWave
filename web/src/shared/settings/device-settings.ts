import type { RejectReasonName } from "../protocol";
import { HOP_LIST_LEN, MAX_SSID_FILTER, type DeviceSettings } from "../protocol";

/** What the device uses until told otherwise; mirrors `Settings::default()` in crates/core. */
export const DEFAULT_DEVICE_SETTINGS: DeviceSettings = {
  scan: { scanAtBoot: false, scanIntervalMs: 250, dedupRssiDb: 6, ssidFilter: "", channelFilterMask: 0 },
  radio: { defaultChannel: 0, hopDwellMs: 250, hopOrder: "ascending", customHopList: Array(HOP_LIST_LEN).fill(0) },
  telemetry: { statusPeriodMs: 1000, logLevel: "info" },
  alerts: { deauthThreshold: 10, deauthWindowMs: 1000 },
};

export type SectionId = "scan" | "radio" | "telemetry" | "alerts";

/** One setting, addressed as `<section>.<field>`. */
export type FieldPath =
  | "scan.scanAtBoot"
  | "scan.scanIntervalMs"
  | "scan.dedupRssiDb"
  | "scan.ssidFilter"
  | "scan.channelFilterMask"
  | "radio.defaultChannel"
  | "radio.hopDwellMs"
  | "radio.hopOrder"
  | "radio.customHopList"
  | "telemetry.statusPeriodMs"
  | "telemetry.logLevel"
  | "alerts.deauthThreshold"
  | "alerts.deauthWindowMs";

export const FIELD_PATHS: readonly FieldPath[] = [
  "scan.scanAtBoot", "scan.scanIntervalMs", "scan.dedupRssiDb", "scan.ssidFilter", "scan.channelFilterMask",
  "radio.defaultChannel", "radio.hopDwellMs", "radio.hopOrder", "radio.customHopList",
  "telemetry.statusPeriodMs", "telemetry.logLevel",
  "alerts.deauthThreshold", "alerts.deauthWindowMs",
];

export const sectionOf = (p: FieldPath): SectionId => p.split(".")[0] as SectionId;

export function getField(s: DeviceSettings, p: FieldPath): unknown {
  const [group, key] = p.split(".") as [SectionId, string];
  return (s[group] as unknown as Record<string, unknown>)[key];
}

/** A copy of `s` with one field changed. */
export function withField(s: DeviceSettings, p: FieldPath, value: unknown): DeviceSettings {
  const [group, key] = p.split(".") as [SectionId, string];
  return { ...s, [group]: { ...s[group], [key]: value } } as DeviceSettings;
}

export const cloneSettings = (s: DeviceSettings): DeviceSettings => JSON.parse(JSON.stringify(s));

/** Ranges the device enforces (crates/core/src/features/settings/validate.rs). */
export const RANGES = {
  scanIntervalMs: [100, 10_000],
  dedupRssiDb: [1, 30],
  hopDwellMs: [100, 2000],
  statusPeriodMs: [250, 5000],
  deauthThreshold: [1, 100],
  deauthWindowMs: [200, 5000],
} as const;

const inRange = (v: number, [lo, hi]: readonly [number, number]) => Number.isInteger(v) && v >= lo && v <= hi;
export const CHANNEL_MASK_BITS = (1 << 13) - 1;

/** A device refusal reason, or a problem only the client can see (the device type cannot hold a longer filter). */
export type ReasonName = RejectReasonName | "ssidTooLong";

export interface Violation {
  field: FieldPath;
  reason: ReasonName;
}

/**
 * The same checks the device makes, so a bad value is caught before it is sent. The device stays the
 * authority: it validates again and refuses a bad set without writing anything. First problem wins, in the
 * same order as the core.
 */
export function validateDeviceSettings(s: DeviceSettings): Violation | null {
  if (!inRange(s.scan.scanIntervalMs, RANGES.scanIntervalMs)) return { field: "scan.scanIntervalMs", reason: "scanIntervalRange" };
  if (!inRange(s.scan.dedupRssiDb, RANGES.dedupRssiDb)) return { field: "scan.dedupRssiDb", reason: "dedupRssiRange" };
  if (!Number.isInteger(s.scan.channelFilterMask) || s.scan.channelFilterMask < 0 || (s.scan.channelFilterMask & ~CHANNEL_MASK_BITS) !== 0) {
    return { field: "scan.channelFilterMask", reason: "channelMaskRange" };
  }
  const ch = s.radio.defaultChannel;
  if (!Number.isInteger(ch) || ch < 0 || ch > 13) return { field: "radio.defaultChannel", reason: "defaultChannelRange" };
  if (!inRange(s.radio.hopDwellMs, RANGES.hopDwellMs)) return { field: "radio.hopDwellMs", reason: "hopDwellRange" };
  if (s.radio.hopOrder === "custom") {
    const list = s.radio.customHopList.filter((c) => c !== 0);
    if (list.some((c, i) => !Number.isInteger(c) || c < 1 || c > 13 || list.indexOf(c) !== i)) {
      return { field: "radio.customHopList", reason: "hopListInvalid" };
    }
    if (list.length === 0) return { field: "radio.customHopList", reason: "hopListEmpty" };
  }
  if (!inRange(s.telemetry.statusPeriodMs, RANGES.statusPeriodMs)) return { field: "telemetry.statusPeriodMs", reason: "statusPeriodRange" };
  if (!inRange(s.alerts.deauthThreshold, RANGES.deauthThreshold)) return { field: "alerts.deauthThreshold", reason: "deauthThresholdRange" };
  if (!inRange(s.alerts.deauthWindowMs, RANGES.deauthWindowMs)) return { field: "alerts.deauthWindowMs", reason: "deauthWindowRange" };
  if (s.alerts.deauthWindowMs < 2 * s.scan.scanIntervalMs) return { field: "alerts.deauthWindowMs", reason: "windowTooShort" };
  if (new TextEncoder().encode(s.scan.ssidFilter).length > MAX_SSID_FILTER) return { field: "scan.ssidFilter", reason: "ssidTooLong" };
  return null;
}

/** The field a device refusal points at (for highlighting). `busy` and `storage` are not about one field. */
export const FIELD_OF_REASON: Readonly<Record<RejectReasonName, FieldPath | null>> = {
  scanIntervalRange: "scan.scanIntervalMs",
  dedupRssiRange: "scan.dedupRssiDb",
  channelMaskRange: "scan.channelFilterMask",
  defaultChannelRange: "radio.defaultChannel",
  hopDwellRange: "radio.hopDwellMs",
  hopListEmpty: "radio.customHopList",
  hopListInvalid: "radio.customHopList",
  statusPeriodRange: "telemetry.statusPeriodMs",
  deauthThresholdRange: "alerts.deauthThreshold",
  deauthWindowRange: "alerts.deauthWindowMs",
  windowTooShort: "alerts.deauthWindowMs",
  busy: null,
  storage: null,
};

export const REASON_TEXT: Readonly<Record<ReasonName, string>> = {
  ssidTooLong: "The SSID filter may be at most 32 bytes.",
  scanIntervalRange: "Scan interval must be 100 to 10000 ms.",
  dedupRssiRange: "Re-report threshold must be 1 to 30 dB.",
  channelMaskRange: "Channel filter may only use channels 1 to 13.",
  defaultChannelRange: "Default channel must be AUTO or 1 to 13.",
  hopDwellRange: "Hop dwell must be 100 to 2000 ms.",
  hopListEmpty: "A custom hop order needs at least one channel.",
  hopListInvalid: "A custom hop order may list channels 1 to 13 once each.",
  statusPeriodRange: "Status period must be 250 to 5000 ms.",
  deauthThresholdRange: "Deauth threshold must be 1 to 100 per second.",
  deauthWindowRange: "Deauth window must be 200 to 5000 ms.",
  windowTooShort: "Deauth window is too short: it must be at least twice the scan interval.",
  busy: "The device is busy (an update is in progress). Try again afterwards.",
  storage: "The device could not write to flash. Nothing was changed.",
};
