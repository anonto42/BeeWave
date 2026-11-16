import { describe, expect, it } from "vitest";
import { OtaUploader } from "../../../src/features/ota/uploader";
import type { Command } from "../../../src/shared/protocol";

/** Mirrors the device's OtaSession (crates/core): sequential offsets, ack = next expected offset. */
class FakeDevice {
  data: number[] = [];
  private size = 0;
  private next = 0;
  private count = 0;
  finished = false;
  signature?: Uint8Array;
  reject = false;
  uploader!: OtaUploader;
  constructor(private readonly dropEvery = 0) {}

  send = async (cmd: Command): Promise<void> => {
    if (cmd.type === "otaBegin") {
      this.size = cmd.size;
      this.next = 0;
      this.data = [];
      this.ack(0);
    } else if (cmd.type === "otaChunk") {
      this.count++;
      if (this.dropEvery && this.count % this.dropEvery === 0) return; // lost on the way
      if (cmd.offset === this.next) {
        this.data.push(...cmd.data);
        this.next += cmd.data.length;
      }
      this.ack(this.next);
    } else if (cmd.type === "otaEndSigned" && this.next === this.size) {
      if (this.reject) {
        setTimeout(() => this.uploader.handleError("ota refused: bad signature"), 0);
        return;
      }
      this.signature = cmd.signature;
      this.finished = true;
      this.ack(this.size);
    }
  };

  private ack(n: number) {
    setTimeout(() => this.uploader.handleAck(n), 0);
  }
}

const SIG = new Uint8Array(64).fill(9);
const image = (n: number) => Uint8Array.from({ length: n }, (_, i) => (i * 7) & 0xff);

async function run(dropEvery: number, size: number) {
  const dev = new FakeDevice(dropEvery);
  const up = new OtaUploader(dev.send, { ackTimeoutMs: 40, window: 4, chunkSize: 16 });
  dev.uploader = up;
  const img = image(size);
  const seen: number[] = [];
  await up.upload(img, SIG, (sent) => seen.push(sent));
  return { dev, img, seen };
}

describe("OtaUploader", () => {
  it("uploads an image exactly and reports progress", async () => {
    const { dev, img, seen } = await run(0, 1000);
    expect(dev.data).toEqual([...img]);
    expect(dev.finished).toBe(true);
    expect(seen.at(-1)).toBe(1000);
    expect(dev.signature).toEqual(SIG);
  });

  it("stops at once with the device's reason when it refuses the signature", async () => {
    const dev = new FakeDevice();
    dev.reject = true;
    const up = new OtaUploader(dev.send, { ackTimeoutMs: 5000, window: 4, chunkSize: 16 });
    dev.uploader = up;
    const started = Date.now();
    await expect(up.upload(image(64), SIG, () => {})).rejects.toThrow(/bad signature/);
    expect(Date.now() - started).toBeLessThan(1000); // did not wait for the ack timeout
  });

  it("rejects a signature of the wrong size before sending anything", async () => {
    const sent: Command[] = [];
    const up = new OtaUploader(async (c) => void sent.push(c));
    await expect(up.upload(image(10), new Uint8Array(10), () => {})).rejects.toThrow(/64 bytes/);
    expect(sent).toEqual([]);
  });

  it("recovers from lost chunks", async () => {
    const { dev, img } = await run(5, 600);
    expect(dev.data).toEqual([...img]);
    expect(dev.finished).toBe(true);
  });

  it("fails when the device never answers", async () => {
    const up = new OtaUploader(async () => {}, { ackTimeoutMs: 10 });
    await expect(up.upload(image(10), SIG, () => {})).rejects.toThrow(/did not answer/);
  });
});
