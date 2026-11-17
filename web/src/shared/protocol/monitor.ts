/** BLE advertisement classes, in `AdClass` variant order (crates/protocol/src/types.rs). */
export const AD_CLASSES = ["fastPair", "ibeacon", "proximity", "generic"] as const;
export type AdClassName = (typeof AD_CLASSES)[number];

/** Anomaly kinds, in `AnomalyKind` variant order. */
export const ANOMALY_KINDS = ["newBssidForSsid", "twoChannels", "rssiOutlier", "returned"] as const;
export type AnomalyKindName = (typeof ANOMALY_KINDS)[number];

/** `HopOrder` variants, in wire order. */
export const HOP_ORDERS = ["ascending", "ascending1_6_11", "custom"] as const;
export type HopOrderName = (typeof HOP_ORDERS)[number];

/** `LogLevel` variants, in wire order. */
export const LOG_LEVELS = ["error", "warn", "info", "debug"] as const;
export type LogLevelName = (typeof LOG_LEVELS)[number];

/** `RejectReason` variants, in wire order. */
export const REJECT_REASONS = [
  "scanIntervalRange",
  "dedupRssiRange",
  "channelMaskRange",
  "defaultChannelRange",
  "hopDwellRange",
  "hopListEmpty",
  "hopListInvalid",
  "statusPeriodRange",
  "deauthThresholdRange",
  "deauthWindowRange",
  "windowTooShort",
  "busy",
  "storage",
] as const;
export type RejectReasonName = (typeof REJECT_REASONS)[number];
