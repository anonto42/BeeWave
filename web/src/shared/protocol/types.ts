import type { AdClassName, AnomalyKindName, RejectReasonName } from "./monitor";
import type { DeviceSettings } from "./settings-types";

/** Device -> app messages. Mirrors `TelemetryMessage` in crates/protocol/src/types.rs. */
export type Telemetry =
  | { type: "status"; activeChannel: number; freeHeap: number; cpuLoad: number; dropped: number; scanning: boolean }
  | { type: "scan"; bssid: string; channel: number; rssi: number; ssid: string }
  | { type: "event"; code: number; message: string }
  | { type: "otaAck"; nextOffset: number }
  | { type: "settings"; channel: number; autoScan: boolean }
  | { type: "authChallenge"; nonce: Uint8Array }
  | { type: "authResult"; ok: boolean }
  | { type: "unclaimed" }
  | { type: "deauth"; bssid: string; count: number }
  | { type: "bleStats"; fastPair: number; ibeacon: number; proximity: number; generic: number }
  | { type: "bleSpike"; class: AdClassName; count: number; baseline: number }
  | { type: "anomaly"; kind: AnomalyKindName; bssid: string; detail: number }
  | { type: "monitorConfig"; deauthThreshold: number; bleEnabled: boolean; bleSpikeFactor: number }
  | { type: "settingsAll"; schema: number; revision: number; settings: DeviceSettings }
  | { type: "settingsSaved"; revision: number; ok: boolean; reason: RejectReasonName | null }
  | {
      type: "storageInfo";
      nvsUsed: number;
      nvsFree: number;
      ota0Active: boolean;
      ota1Active: boolean;
      ota0Size: number;
      ota1Size: number;
      settingsRevision: number;
    }
  | { type: "selfTest"; radio: boolean; nvs: boolean; ble: boolean; otaSlot: boolean; busy: boolean }
  | {
      type: "deviceInfo";
      version: string;
      gitSha: string;
      built: string;
      /** 16 bytes as 32 lowercase hex characters. */
      otaKeyFp: string;
      requestConnParams: boolean;
      coexDebug: boolean;
      bonding: boolean;
    }
  | { type: "rejected"; reason: RejectReasonName };

/** App -> device messages. Mirrors `DeviceCommand` in crates/protocol/src/types.rs. */
export type Command =
  | { type: "startScan"; bandMask: number }
  | { type: "stop" }
  | { type: "configureRadio"; channel: number }
  | { type: "reboot" }
  | { type: "otaBegin"; size: number; sha256: Uint8Array }
  | { type: "otaChunk"; offset: number; data: Uint8Array }
  | { type: "otaEnd" }
  | { type: "getSettings" }
  | { type: "setSettings"; channel: number; autoScan: boolean }
  | { type: "authBegin" }
  | { type: "authProve"; mac: Uint8Array }
  | { type: "otaEndSigned"; signature: Uint8Array }
  | { type: "provision"; key: Uint8Array }
  | { type: "getMonitor" }
  | { type: "setMonitor"; deauthThreshold: number; bleEnabled: boolean; bleSpikeFactor: number }
  | { type: "getSettingsAll" }
  | { type: "saveSettingsAll"; settings: DeviceSettings }
  | { type: "factoryReset" }
  | { type: "runSelfTest" }
  | { type: "getStorageInfo" }
  | { type: "getDeviceInfo" };

/** A decoded telemetry packet with its sequence number (for loss detection). */
export interface TelemetryPacket {
  seq: number;
  msg: Telemetry;
}
