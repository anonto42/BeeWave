import type { AlertRecord, AlertSource } from "../../shared/monitor";

export type FeedFilter = "all" | AlertSource;

export const FILTERS: Array<[FeedFilter, string]> = [
  ["all", "ALL"],
  ["anomaly", "ANOMALY"],
  ["deauth", "DEAUTH"],
  ["ble", "BLE"],
  ["beaconFlood", "FLOOD"],
  ["watchlist", "WATCH"],
];

/** The alerts to show, newest first (the store already keeps that order). */
export const visible = (alerts: readonly AlertRecord[], f: FeedFilter): AlertRecord[] => alerts.filter((a) => f === "all" || a.source === f);
