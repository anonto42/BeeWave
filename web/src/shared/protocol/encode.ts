import { MAX_OTA_CHUNK, PROTO_VERSION, SIGNATURE_LEN } from "./constants";
import { HOP_ORDERS, LOG_LEVELS } from "./monitor";
import { HOP_LIST_LEN, MAX_SSID_FILTER, type DeviceSettings } from "./settings-types";
import type { Command } from "./types";

function varint(n: number): number[] {
  const out: number[] = [];
  do {
    let byte = n % 128;
    n = Math.floor(n / 128);
    if (n) byte |= 0x80;
    out.push(byte);
  } while (n);
  return out;
}

/** postcard encoding of `ScanSettings`, `RadioSettings`, `TelemetrySettings`, `AlertSettings` (field order is the wire format). */
function settingsBody(s: DeviceSettings): number[] {
  const ssid = new TextEncoder().encode(s.scan.ssidFilter);
  if (ssid.length > MAX_SSID_FILTER) throw new Error(`ssid filter longer than ${MAX_SSID_FILTER} bytes`);
  if (s.radio.customHopList.length !== HOP_LIST_LEN) throw new Error(`hop list must have ${HOP_LIST_LEN} slots`);
  return [
    s.scan.scanAtBoot ? 1 : 0,
    ...varint(s.scan.scanIntervalMs),
    s.scan.dedupRssiDb,
    ssid.length,
    ...ssid,
    ...varint(s.scan.channelFilterMask),
    s.radio.defaultChannel,
    ...varint(s.radio.hopDwellMs),
    HOP_ORDERS.indexOf(s.radio.hopOrder),
    ...s.radio.customHopList,
    ...varint(s.telemetry.statusPeriodMs),
    LOG_LEVELS.indexOf(s.telemetry.logLevel),
    s.alerts.deauthThreshold,
    ...varint(s.alerts.deauthWindowMs),
  ];
}

/** Encode one command write. Enum tag order must match `DeviceCommand` in Rust. */
export function encodeCommand(seq: number, c: Command): Uint8Array<ArrayBuffer> {
  const o: number[] = [PROTO_VERSION, seq & 0xff];
  switch (c.type) {
    case "startScan": o.push(0, c.bandMask); break;
    case "stop": o.push(1); break;
    case "configureRadio": o.push(2, c.channel); break;
    case "reboot": o.push(3); break;
    case "otaBegin":
      if (c.sha256.length !== 32) throw new Error("sha256 must be 32 bytes");
      o.push(4, ...varint(c.size), ...c.sha256);
      break;
    case "otaChunk":
      if (c.data.length > MAX_OTA_CHUNK) throw new Error(`chunk > ${MAX_OTA_CHUNK} bytes`);
      o.push(5, ...varint(c.offset), ...varint(c.data.length), ...c.data);
      break;
    case "otaEnd": o.push(6); break;
    case "getSettings": o.push(7); break;
    case "setSettings": o.push(8, c.channel, c.autoScan ? 1 : 0); break;
    case "authBegin": o.push(9); break;
    case "authProve":
      if (c.mac.length !== 32) throw new Error("mac must be 32 bytes");
      o.push(10, ...c.mac);
      break;
    case "otaEndSigned":
      // Ed25519 signature R||S; the two 32-byte halves are two plain arrays in Rust.
      if (c.signature.length !== SIGNATURE_LEN) throw new Error(`signature must be ${SIGNATURE_LEN} bytes`);
      o.push(11, ...c.signature);
      break;
    case "provision":
      if (c.key.length !== 32) throw new Error("key must be 32 bytes");
      o.push(12, ...c.key);
      break;
    case "getMonitor": o.push(13); break;
    case "getSettingsAll": o.push(15); break;
    case "saveSettingsAll": o.push(16, ...settingsBody(c.settings)); break;
    case "factoryReset": o.push(17); break;
    case "runSelfTest": o.push(18); break;
    case "getStorageInfo": o.push(19); break;
    case "getDeviceInfo": o.push(20); break;
    case "setHandshakeCapture": o.push(21, c.enabled ? 1 : 0); break;
    case "getHandshakeCapture": o.push(22); break;
    case "setMonitor": o.push(14, c.deauthThreshold, c.bleEnabled ? 1 : 0, c.bleSpikeFactor); break;
  }
  return Uint8Array.from(o);
}
