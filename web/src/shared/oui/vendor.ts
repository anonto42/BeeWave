import { OUI_VENDORS } from "./table";

/** Bit 1 of the first byte set = locally administered: a random/private address, not a vendor's. */
const LOCALLY_ADMINISTERED = 0x02;

/** "Randomized", a vendor name from the local table, or "Unknown". Accepts `aa:bb:cc:...` or `aabbcc...`. */
export function vendorOf(bssid: string): string {
  const hex = bssid.toLowerCase().replace(/[^0-9a-f]/g, "");
  if (hex.length < 6) return "Unknown";
  if (parseInt(hex.slice(0, 2), 16) & LOCALLY_ADMINISTERED) return "Randomized";
  return OUI_VENDORS[hex.slice(0, 6)] ?? "Unknown";
}
