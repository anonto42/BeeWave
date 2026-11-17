import type { ScanResult } from "./types";

export type SortKey = "rssi" | "channel" | "ssid";

/** A sorted copy: strongest signal first, lowest channel first, or SSID A-Z (hidden networks last). */
export function sortNetworks(list: readonly ScanResult[], key: SortKey): ScanResult[] {
  const out = [...list];
  switch (key) {
    case "rssi":
      return out.sort((a, b) => b.rssi - a.rssi);
    case "channel":
      return out.sort((a, b) => a.channel - b.channel || b.rssi - a.rssi);
    case "ssid":
      return out.sort((a, b) => Number(a.ssid === "") - Number(b.ssid === "") || a.ssid.localeCompare(b.ssid));
  }
}
