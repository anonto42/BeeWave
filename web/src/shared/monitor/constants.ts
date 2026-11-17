/** The device de-duplicates scan results: a known network is reported again only when RSSI moves this much (dB). */
export const RSSI_DELTA_DB = 6;
/** Status messages (1 per second) per scan cycle: about one 13-channel hop lap, matching the device. */
export const CYCLE_TICKS = 4;
/** RSSI points kept per network. */
export const HISTORY_CAP = 60;
/** Reports kept for export and channel activity. */
export const EVENT_CAP = 5000;
/** Saved scans kept for comparison. */
export const SNAPSHOT_CAP = 20;
/** Alerts kept. */
export const ALERT_CAP = 500;
/** Window of the channel activity series. */
export const ACTIVITY_WINDOW_MS = 60_000;
