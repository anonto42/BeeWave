import { HEADER_LEN } from "./constants";

/** Cursor over a packet body, reading postcard primitives. Throws on truncated input. */
export class Reader {
  private i = HEADER_LEN;

  constructor(private readonly buf: Uint8Array) {}

  u8(): number {
    if (this.i >= this.buf.length) throw new Error("short packet");
    return this.buf[this.i++];
  }

  i8(): number {
    const v = this.u8();
    return v > 127 ? v - 256 : v;
  }

  /** postcard varint (LEB128). Uses multiplication, not <<, to stay correct above 2^31. */
  varint(): number {
    let result = 0;
    let shift = 0;
    for (;;) {
      const byte = this.u8();
      result += (byte & 0x7f) * 2 ** shift;
      if (!(byte & 0x80)) return result;
      shift += 7;
      if (shift > 35) throw new Error("varint too long");
    }
  }

  bytes(n: number): Uint8Array {
    if (this.i + n > this.buf.length) throw new Error("short packet");
    const out = this.buf.subarray(this.i, this.i + n);
    this.i += n;
    return out;
  }

  /** Length-prefixed UTF-8 string. */
  str(): string {
    return new TextDecoder().decode(this.bytes(this.varint()));
  }
}
