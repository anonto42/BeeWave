/** Device -> app messages. Mirrors `TelemetryMessage` in crates/protocol/src/types.rs. */
export type Telemetry =
  | { type: "status"; activeChannel: number; freeHeap: number; cpuLoad: number; dropped: number }
  | { type: "scan"; bssid: string; channel: number; rssi: number; ssid: string }
  | { type: "event"; code: number; message: string }
  | { type: "otaAck"; nextOffset: number }
  | { type: "settings"; channel: number; autoScan: boolean }
  | { type: "authChallenge"; nonce: Uint8Array }
  | { type: "authResult"; ok: boolean };

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
  | { type: "otaEndSigned"; signature: Uint8Array };

/** A decoded telemetry packet with its sequence number (for loss detection). */
export interface TelemetryPacket {
  seq: number;
  msg: Telemetry;
}
