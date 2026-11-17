export type RssiUnit = "dbm" | "bars";

let unit: RssiUnit = "dbm";

/** Set by the phone-side "RSSI unit" setting; the lists and tables read it when they draw. */
export function setRssiUnit(u: RssiUnit): void {
  unit = u;
}

export const currentRssiUnit = (): RssiUnit => unit;

/** 0-5 bars from the usual bands: -30 and above excellent, -67 very good, -70 good, -80 fair, -90 poor, below that none. */
export function rssiLevel(rssi: number): 0 | 1 | 2 | 3 | 4 | 5 {
  if (rssi >= -30) return 5;
  if (rssi >= -67) return 4;
  if (rssi >= -70) return 3;
  if (rssi >= -80) return 2;
  if (rssi >= -90) return 1;
  return 0;
}

/** The RSSI as the owner wants to read it: the dBm number, or five bars (with the number as a tooltip). */
export function rssiHtml(rssi: number, u: RssiUnit = unit): string {
  if (u === "dbm") return `<span class="dbm">${rssi}</span>`;
  return `<span class="sig" data-bars="${rssiLevel(rssi)}" title="${rssi} dBm" role="img" aria-label="${rssiLevel(rssi)} of 5 bars"><i></i><i></i><i></i><i></i><i></i></span>`;
}
