import type { ScanResult } from "./types";

export type FilterKey = "all" | "-80" | "-70" | "-60" | "stations";

/**
 * Filter networks by minimum RSSI threshold or presence of associated client stations.
 */
export function filterNetworks(
  list: readonly ScanResult[],
  key: FilterKey,
  stationCountFn?: (bssid: string) => number,
): ScanResult[] {
  switch (key) {
    case "-80":
      return list.filter((n) => n.rssi >= -80);
    case "-70":
      return list.filter((n) => n.rssi >= -70);
    case "-60":
      return list.filter((n) => n.rssi >= -60);
    case "stations":
      return list.filter((n) => (stationCountFn ? stationCountFn(n.bssid) > 0 : false));
    case "all":
    default:
      return [...list];
  }
}
