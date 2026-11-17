/** Signal strength as 1-4 bars from the RSSI in dBm. */
export function signalBars(rssi: number): 1 | 2 | 3 | 4 {
  if (rssi >= -55) return 4;
  if (rssi >= -67) return 3;
  if (rssi >= -77) return 2;
  return 1;
}
