import type { Telemetry } from "../protocol";
import { vendorOf } from "../oui";
import type { NewAlert } from "./alert-store";

const ANOMALY_TEXT = {
  newBssidForSsid: { severity: "warn", title: "New access point for a known network", detail: "same name, different BSSID (a second AP, or a look-alike)" },
  twoChannels: { severity: "info", title: "Same BSSID on two channels", detail: "may be adjacent-channel leakage; a hint, not proof" },
  rssiOutlier: { severity: "info", title: "Signal jumped", detail: "" },
  returned: { severity: "info", title: "Network came back", detail: "" },
} as const;

/** A deauth burst this size or bigger is treated as a flood (red); smaller bursts are yellow. */
export const DEAUTH_FLOOD = 50;
/** A BLE class this many times above its baseline is treated as a flood (red). */
export const BLE_FLOOD_FACTOR = 10;

const CLASS_LABEL = { fastPair: "Fast Pair", ibeacon: "iBeacon", proximity: "Proximity pairing", generic: "Generic" } as const;

/** The alert a device message stands for, or null if the message is not an alert. */
export function alertFromTelemetry(m: Telemetry): NewAlert | null {
  switch (m.type) {
    case "deauth":
      return {
        source: "deauth",
        severity: m.count >= DEAUTH_FLOOD ? "alert" : "warn",
        title: `Deauth frames: ${m.count}/s`,
        detail: `${m.bssid} (${vendorOf(m.bssid)})`,
        bssid: m.bssid,
      };
    case "bleSpike":
      return {
        source: "ble",
        severity: m.count >= Math.max(1, m.baseline) * BLE_FLOOD_FACTOR ? "alert" : "warn",
        title: `${CLASS_LABEL[m.class]} advertisements spiked`,
        detail: `${m.count} in the last minute, baseline ${m.baseline}`,
      };
    case "handshakeComplete":
      return {
        source: "handshake",
        severity: "info",
        title: "Handshake captured",
        detail: `${m.bssid} <-> ${m.client}: enough messages for an offline attempt (subject to any truncated message)`,
        bssid: m.bssid,
      };
    case "anomaly": {
      const t = ANOMALY_TEXT[m.kind];
      const extra =
        m.kind === "rssiOutlier" ? `${m.detail} dB between cycles` : m.kind === "returned" ? `after ${m.detail} cycles` : t.detail;
      return { source: "anomaly", severity: t.severity, title: t.title, detail: `${m.bssid} (${vendorOf(m.bssid)}): ${extra}`, bssid: m.bssid };
    }
    case "beaconFloodAlert":
      return {
        source: "beaconFlood",
        severity: "alert",
        title: `Beacon flood attack: ${m.rate}/s`,
        detail: `${m.uniqueBssids} unique BSSIDs in 1s window (possible Hydra-ESP / Marauder beacon spam)`,
      };
    case "decloakedResult":
      return {
        source: "decloaked",
        severity: "warn",
        title: `Hidden network uncloaked: "${m.ssid}"`,
        detail: `${m.bssid} (${vendorOf(m.bssid)}) on CH ${m.channel} · ${m.rssi} dBm`,
        bssid: m.bssid,
      };
    default:
      return null;
  }
}
