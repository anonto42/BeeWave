import { HEADER_LEN, PROTO_VERSION } from "./constants";
import { AD_CLASSES, ANOMALY_KINDS } from "./monitor";
import { Reader } from "./reader";
import type { Telemetry, TelemetryPacket } from "./types";

/** The name of an enum variant by its wire index; an unknown index is a decode error, never a guess. */
function enumName<T extends string>(names: readonly T[], i: number): T {
  const name = names[i];
  if (name === undefined) throw new Error(`unknown enum variant ${i}`);
  return name;
}

const hex = (b: Uint8Array) => [...b].map((x) => x.toString(16).padStart(2, "0")).join(":");

/** Decode one notification. Enum tag order must match `TelemetryMessage` in Rust. */
export function decodeTelemetry(buf: Uint8Array): TelemetryPacket {
  if (buf.length < HEADER_LEN) throw new Error("short packet");
  if (buf[0] !== PROTO_VERSION) throw new Error(`bad protocol version ${buf[0]}`);

  const r = new Reader(buf);
  const tag = r.varint();
  let msg: Telemetry;
  switch (tag) {
    case 0:
      msg = { type: "status", activeChannel: r.u8(), freeHeap: r.varint(), cpuLoad: r.u8(), dropped: r.varint(), scanning: r.u8() !== 0 };
      break;
    case 1:
      msg = { type: "scan", bssid: hex(r.bytes(6)), channel: r.u8(), rssi: r.i8(), ssid: r.str() };
      break;
    case 2:
      msg = { type: "event", code: r.varint(), message: r.str() };
      break;
    case 3:
      msg = { type: "otaAck", nextOffset: r.varint() };
      break;
    case 4:
      msg = { type: "settings", channel: r.u8(), autoScan: r.u8() !== 0 };
      break;
    case 5:
      msg = { type: "authChallenge", nonce: r.bytes(16).slice() };
      break;
    case 6:
      msg = { type: "authResult", ok: r.u8() !== 0 };
      break;
    case 7:
      msg = { type: "unclaimed" };
      break;
    case 8:
      msg = { type: "deauth", bssid: hex(r.bytes(6)), count: r.varint() };
      break;
    case 9:
      msg = { type: "bleStats", fastPair: r.varint(), ibeacon: r.varint(), proximity: r.varint(), generic: r.varint() };
      break;
    case 10:
      msg = { type: "bleSpike", class: enumName(AD_CLASSES, r.varint()), count: r.varint(), baseline: r.varint() };
      break;
    case 11:
      msg = { type: "anomaly", kind: enumName(ANOMALY_KINDS, r.varint()), bssid: hex(r.bytes(6)), detail: r.u8() };
      break;
    case 12:
      msg = { type: "monitorConfig", deauthThreshold: r.u8(), bleEnabled: r.u8() !== 0, bleSpikeFactor: r.u8() };
      break;
    default:
      throw new Error(`unknown telemetry tag ${tag}`);
  }
  return { seq: buf[1], msg };
}
