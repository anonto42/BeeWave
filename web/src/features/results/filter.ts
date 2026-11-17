import type { NetworkRecord } from "../../shared/monitor";
import { vendorOf } from "../../shared/oui";

export type SortKey = "ssid" | "bssid" | "vendor" | "rssi" | "channel" | "firstSeen" | "lastSeen" | "count";
export type SortDir = "asc" | "desc";

export interface ResultRow {
  record: NetworkRecord;
  vendor: string;
}

export interface ResultQuery {
  sort: SortKey;
  dir: SortDir;
  /** Case-insensitive substring of the SSID ("" = all). */
  ssid: string;
  /** 1-13, or 0 for all channels. */
  channel: number;
}

export const DEFAULT_QUERY: ResultQuery = { sort: "rssi", dir: "desc", ssid: "", channel: 0 };

const value = (r: ResultRow, k: SortKey): string | number => (k === "vendor" ? r.vendor : k === "ssid" ? r.record.ssid : r.record[k]);

/** Filter by SSID substring and channel, then sort by any column (text columns ignore case). */
export function filterAndSort(records: readonly NetworkRecord[], q: ResultQuery): ResultRow[] {
  const needle = q.ssid.trim().toLowerCase();
  const rows = records
    .filter((r) => (!needle || r.ssid.toLowerCase().includes(needle)) && (!q.channel || r.channel === q.channel))
    .map((record) => ({ record, vendor: vendorOf(record.bssid) }));
  const sign = q.dir === "asc" ? 1 : -1;
  return rows.sort((a, b) => {
    const x = value(a, q.sort);
    const y = value(b, q.sort);
    const c = typeof x === "number" && typeof y === "number" ? x - y : String(x).localeCompare(String(y), undefined, { sensitivity: "base" });
    return c * sign || a.record.bssid.localeCompare(b.record.bssid);
  });
}

/** Clicking a column: the same column flips direction, a new one starts descending for numbers and ascending for text. */
export function nextQuery(q: ResultQuery, key: SortKey): ResultQuery {
  if (q.sort === key) return { ...q, dir: q.dir === "asc" ? "desc" : "asc" };
  const text = key === "ssid" || key === "bssid" || key === "vendor";
  return { ...q, sort: key, dir: text ? "asc" : "desc" };
}
