import type { GpsCoordinates } from "../gps";
import type { NetworkRecord, SecurityRecord } from "./types";

/**
 * Maps a network's detected security posture to standard WiGLE AuthMode capability tags.
 */
export function networkToWigleAuthMode(sec?: SecurityRecord): string {
  if (!sec) return "[ESS]";
  let base = "";
  switch (sec.protocol) {
    case "open":
      base = "[OPEN][ESS]";
      break;
    case "wep":
      base = "[WEP][ESS]";
      break;
    case "wpa":
      base = "[WPA-PSK-TKIP][ESS]";
      break;
    case "wpa2":
      base = "[WPA2-PSK-CCMP][RSN][ESS]";
      break;
    case "wpa3":
      base = "[WPA3-SAE-CCMP][RSN][ESS]";
      break;
    case "wpa2Wpa3":
      base = "[WPA2-PSK-CCMP][WPA3-SAE-CCMP][RSN][ESS]";
      break;
    case "owe":
      base = "[OWE-CCMP][RSN][ESS]";
      break;
    case "enterprise":
      base = "[WPA2-EAP-CCMP][RSN][ESS]";
      break;
    default:
      base = "[ESS]";
  }
  if (sec.pmf === "required") base += "[PMF-R]";
  else if (sec.pmf === "capable") base += "[PMF-C]";
  return base;
}

/**
 * Returns the center frequency in MHz for a 2.4 GHz 802.11 channel.
 */
export function channelToFrequencyMhz(channel: number): number {
  if (channel === 14) return 2484;
  if (channel >= 1 && channel <= 13) return 2407 + channel * 5;
  return 2412;
}

/**
 * Formats a UNIX timestamp (ms) as WiGLE UTC format: `YYYY-MM-DD HH:MM:SS`.
 */
export function formatWigleTimestamp(epochMs: number): string {
  const d = new Date(epochMs);
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())} ${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}:${pad(d.getUTCSeconds())}`;
}

export function escapeCsvField(val: string): string {
  if (val.includes('"') || val.includes(",") || val.includes("\n") || val.includes("\r")) {
    return `"${val.replace(/"/g, '""')}"`;
  }
  return val;
}

/**
 * Builds a standard WiGLE CSV v1.4 export string from observed networks.
 */
export function generateWigleCsv(networks: readonly NetworkRecord[], defaultGps?: GpsCoordinates | null): string {
  const lines = [
    "WigleWifi-1.4,appRelease=2.0.0,model=BeeWave-ESP32,release=2.0.0,device=BeeWave,display=BeeWave,board=ESP32,brand=BeeWave",
    "MAC,SSID,AuthMode,FirstSeen,Channel,Frequency,RSSI,CurrentLatitude,CurrentLongitude,AltitudeMeters,AccuracyMeters,Type",
  ];

  for (const n of networks) {
    const gps = n.gps ?? defaultGps;
    const lat = gps ? gps.lat.toFixed(6) : "0.000000";
    const lon = gps ? gps.lon.toFixed(6) : "0.000000";
    const alt = gps ? gps.alt.toFixed(1) : "0.0";
    const acc = gps ? gps.accuracy.toFixed(1) : "0.0";
    const auth = networkToWigleAuthMode(n.security);
    const freq = channelToFrequencyMhz(n.channel);
    const time = formatWigleTimestamp(n.firstSeen);
    const ssid = escapeCsvField(n.ssid);

    lines.push(`${n.bssid},${ssid},${auth},${time},${n.channel},${freq},${n.rssi},${lat},${lon},${alt},${acc},WIFI`);
  }

  return lines.join("\r\n") + "\r\n";
}
