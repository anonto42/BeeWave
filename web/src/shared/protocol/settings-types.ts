import type { HopOrderName, LogLevelName } from "./monitor";

/** Version of the settings layout this client understands (`SETTINGS_SCHEMA` in crates/protocol). */
export const SETTINGS_SCHEMA = 1;
export const MAX_SSID_FILTER = 32;
export const HOP_LIST_LEN = 13;

export interface ScanSettings {
  scanAtBoot: boolean;
  scanIntervalMs: number;
  dedupRssiDb: number;
  ssidFilter: string;
  channelFilterMask: number;
}

export interface RadioSettings {
  defaultChannel: number;
  hopDwellMs: number;
  hopOrder: HopOrderName;
  /** 13 slots: channels 1-13 in visiting order, unused slots 0. */
  customHopList: number[];
}

export interface TelemetrySettings {
  statusPeriodMs: number;
  logLevel: LogLevelName;
}

export interface AlertSettings {
  deauthThreshold: number;
  deauthWindowMs: number;
}

/** Every setting stored on the device, in the four groups of the wire protocol. */
export interface DeviceSettings {
  scan: ScanSettings;
  radio: RadioSettings;
  telemetry: TelemetrySettings;
  alerts: AlertSettings;
}
