import type { GpsCoordinates } from "../gps";
import type { WifiPmfMode, WifiSecurityProtocol } from "../protocol";

export interface SecurityRecord {
  protocol: WifiSecurityProtocol;
  pmf: WifiPmfMode;
}

export interface WpsRecord {
  version: number;
  state: number;
  locked: boolean;
  selectedRegistrar: boolean;
  deviceName: string;
  manufacturer: string;
  modelName: string;
}

/** One network as the phone has seen it. */
export interface NetworkRecord {
  bssid: string;
  ssid: string;
  channel: number;
  /** Latest RSSI in dBm. */
  rssi: number;
  /** Milliseconds since the epoch. */
  firstSeen: number;
  lastSeen: number;
  /** How many reports the device sent for it (the device already de-duplicates, see below). */
  count: number;
  /** RSSI over the scan cycles in which the device reported it (oldest first, capped). */
  history: RssiPoint[];
  wps?: WpsRecord;
  security?: SecurityRecord;
  decloaked?: boolean;
  gps?: GpsCoordinates;
}

export interface RssiPoint {
  cycle: number;
  rssi: number;
  at: number;
}

/** A network in a saved scan (what the comparison needs). */
export interface SnapshotNetwork {
  bssid: string;
  ssid: string;
  channel: number;
  rssi: number;
}

export interface Snapshot {
  id: number;
  label: string;
  at: number;
  networks: SnapshotNetwork[];
}

export type Severity = "info" | "warn" | "alert";
export type AlertSource = "deauth" | "ble" | "anomaly" | "watchlist" | "handshake" | "beaconFlood" | "decloaked" | "bleFlood";

export interface AlertRecord {
  id: number;
  at: number;
  source: AlertSource;
  severity: Severity;
  title: string;
  detail: string;
  bssid?: string;
}

/** One report from the device, kept for export and channel activity. */
export interface ScanEvent {
  at: number;
  bssid: string;
  ssid: string;
  channel: number;
  rssi: number;
  gps?: GpsCoordinates;
}
