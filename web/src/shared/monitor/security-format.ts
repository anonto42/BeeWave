import type { SecurityRecord } from "./types";

const PROTO_MAP: Record<string, string> = {
  open: "OPEN",
  wep: "WEP",
  wpa: "WPA",
  wpa2: "WPA2",
  wpa3: "WPA3",
  wpa2Wpa3: "WPA2/WPA3",
  owe: "OWE",
  enterprise: "802.1X",
};

/** Format a SecurityRecord as a readable human badge text (e.g. "WPA2 · PMF" or "OPEN"). */
export const securityLabel = (sec: SecurityRecord): string => {
  const proto = PROTO_MAP[sec.protocol] ?? sec.protocol.toUpperCase();
  const pmf = sec.pmf === "required" ? " · PMF" : sec.pmf === "capable" ? " · PMF OPT" : "";
  return `${proto}${pmf}`;
};
