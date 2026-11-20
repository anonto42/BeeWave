/** BLE advertisement classes, in `AdClass` variant order (crates/protocol/src/types.rs). */
export const AD_CLASSES = ["fastPair", "ibeacon", "proximity", "generic", "findMy"] as const;
export type AdClassName = (typeof AD_CLASSES)[number];

/** Anomaly kinds, in `AnomalyKind` variant order. */
export const ANOMALY_KINDS = [
  "newBssidForSsid",
  "twoChannels",
  "rssiOutlier",
  "returned",
  "securityDowngrade",
  "karmaSsidChange",
  "tsfRollback",
] as const;
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

/** `WifiSecurityProtocol` variants, in wire order. */
export const WIFI_SECURITY_PROTOCOLS = [
  "open",
  "wep",
  "wpa",
  "wpa2",
  "wpa3",
  "wpa2Wpa3",
  "owe",
  "enterprise",
] as const;
export type WifiSecurityProtocol = (typeof WIFI_SECURITY_PROTOCOLS)[number];

/** `WifiPmfMode` variants, in wire order. */
export const WIFI_PMF_MODES = ["none", "capable", "required"] as const;
export type WifiPmfMode = (typeof WIFI_PMF_MODES)[number];

/** `WifiPhyStandard` variants, in wire order. */
export const WIFI_PHY_STANDARDS = ["unknown", "b", "g", "n", "ax"] as const;
export type WifiPhyStandard = (typeof WIFI_PHY_STANDARDS)[number];
