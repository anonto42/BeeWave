import { MAX_OTA_CHUNK, PROTO_VERSION } from "./constants";
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
  }
  return Uint8Array.from(o);
}
