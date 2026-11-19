import type { Telemetry } from "../protocol";
import { vendorOf } from "../oui";
import type { NewAlert } from "./alert-store";

const ANOMALY_TEXT = {
  newBssidForSsid: { severity: "warn", title: "New access point for a known network", detail: "same name, different BSSID (a second AP, or a look-alike)" },
  twoChannels: { severity: "info", title: "Same BSSID on two channels", detail: "may be adjacent-channel leakage; a hint, not proof" },
  rssiOutlier: { severity: "info", title: "Signal jumped", detail: "" },
  returned: { severity: "info", title: "Network came back", detail: "" },
  securityDowngrade: { severity: "alert", title: "Evil Twin security downgrade detected", detail: "open or downgraded access point clone detected for secured network" },
} as const;

/** A deauth burst this size or bigger is treated as a flood (red); smaller bursts are yellow. */
export const DEAUTH_FLOOD = 50;
/** A BLE class this many times above its baseline is treated as a flood (red). */
export const BLE_FLOOD_FACTOR = 10;

const CLASS_LABEL = {
  fastPair: "Fast Pair",
  ibeacon: "iBeacon",
  proximity: "Proximity pairing",
  generic: "Generic",
  findMy: "Apple Find My (AirTag)",
} as const;

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
    case "disassocAlert":
      return {
        source: "disassoc",
        severity: m.count >= DEAUTH_FLOOD ? "alert" : "warn",
        title: `Disassoc frames: ${m.count}/s`,
        detail: `${m.bssid} (${vendorOf(m.bssid)}) (possible aireplay-ng / Marauder kick attack)`,
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
    case "bleFloodAlert":
      return {
        source: "bleFlood",
        severity: "alert",
        title: `BLE advertisement flood: ${m.rate}/s`,
        detail: `${CLASS_LABEL[m.class]} burst (possible Flipper Zero / Hydra-ESP BLE spam)`,
      };
    case "probeFloodAlert":
      return {
        source: "probeFlood",
        severity: "alert",
        title: `Probe request flood attack: ${m.rate}/s`,
        detail: `${m.uniqueClients} unique client MACs in 1s window (possible mdk4 / Marauder probe spam)`,
      };
    default:
      return null;
  }
}
