/** BLE advertisement classes, in `AdClass` variant order (crates/protocol/src/types.rs). */
export const AD_CLASSES = ["fastPair", "ibeacon", "proximity", "generic"] as const;
export type AdClassName = (typeof AD_CLASSES)[number];

/** Anomaly kinds, in `AnomalyKind` variant order. */
export const ANOMALY_KINDS = ["newBssidForSsid", "twoChannels", "rssiOutlier", "returned"] as const;
export type AnomalyKindName = (typeof ANOMALY_KINDS)[number];
