import { MAX_OTA_CHUNK, SIGNATURE_LEN, type Command } from "../../shared/protocol";
import { sha256 } from "./sha256";
import type { OtaOptions, OtaProgress } from "./types";

const DEFAULTS: OtaOptions = { window: 8, chunkSize: MAX_OTA_CHUNK, ackTimeoutMs: 3000, maxRetries: 5 };

/**
 * Sends a firmware image with a sliding window. The device acknowledges with the offset it expects
 * next (`OtaAck`), so a lost or reordered chunk shows up as an ack that does not advance and we
 * rewind to the acknowledged offset and resend.
 */
export class OtaUploader {
  private acked = -1; // -1 = no ack yet
  private ackSeq = 0; // bumps on every ack, including repeats
  private waiter?: () => void;
  private failure?: string; // device-reported error, ends the upload immediately
  private readonly opts: OtaOptions;

  constructor(
    private readonly send: (cmd: Command) => Promise<void>,
    opts: Partial<OtaOptions> = {},
  ) {
    this.opts = { ...DEFAULTS, ...opts };
  }

  /** Feed every `otaAck` telemetry message here. */
  handleAck(nextOffset: number): void {
    this.acked = nextOffset;
    this.ackSeq++;
    this.waiter?.();
  }

  /** Feed OTA error events here (the device refused or failed): the upload stops with that reason. */
  handleError(message: string): void {
    this.failure = message;
    this.waiter?.();
  }

  /** `signature` is the 64-byte Ed25519 signature of the image's SHA-256 (`beewave-app.bin.sig`). */
  async upload(image: Uint8Array<ArrayBuffer>, signature: Uint8Array, onProgress: OtaProgress): Promise<void> {
    if (signature.length !== SIGNATURE_LEN) throw new Error(`signature file must be ${SIGNATURE_LEN} bytes`);
    this.failure = undefined;
    const total = image.length;
    const { window, chunkSize } = this.opts;
    const hash = await sha256(image);

    this.acked = -1;
    await this.send({ type: "otaBegin", size: total, sha256: hash });
    if (!(await this.waitForAck(() => this.acked >= 0))) throw new Error("device did not answer OTA begin");

    let sent = 0;
    let retries = 0;
    let rewound = false; // after a rewind, ignore repeated acks from chunks already in flight
    while (this.acked < total) {
      const seq = this.ackSeq; // snapshot first so acks arriving while we send are not missed
      const before = this.acked;
      while (sent < total && sent - this.acked < window * chunkSize) {
        const end = Math.min(sent + chunkSize, total);
        await this.send({ type: "otaChunk", offset: sent, data: image.subarray(sent, end) });
        sent = end;
      }

      const gotAck = await this.waitForAck(() => this.ackSeq !== seq);
      if (!gotAck) {
        if (++retries > this.opts.maxRetries) throw new Error("OTA stalled: no acks from device");
        sent = Math.max(this.acked, 0); // timeout: resend from the last acknowledged offset
        rewound = true;
      } else if (this.acked > before) {
        retries = 0;
        rewound = false;
      } else if (!rewound) {
        sent = Math.max(this.acked, 0); // device repeated its offset: it dropped a chunk
        rewound = true;
      }
      onProgress(Math.max(this.acked, 0), total);
    }

    // The last chunk's ack already says `total`, so wait for a *new* ack: the device's answer to the
    // signed end (sent only after it verified the signature and finalised the image).
    const seq = this.ackSeq;
    await this.send({ type: "otaEndSigned", signature });
    if (!(await this.waitForAck(() => this.ackSeq !== seq && this.acked === total))) {
      throw new Error("device did not confirm the update");
    }
    onProgress(total, total);
  }

  /** Resolve true once `done()` holds, false after the ack timeout. */
  private async waitForAck(done: () => boolean): Promise<boolean> {
    const ok = await this.waitRaw(() => done() || this.failure !== undefined);
    if (this.failure !== undefined) throw new Error(this.failure);
    return ok;
  }

  private waitRaw(done: () => boolean): Promise<boolean> {
    if (done()) return Promise.resolve(true);
    return new Promise((resolve) => {
      const timer = setTimeout(() => {
        this.waiter = undefined;
        resolve(false);
      }, this.opts.ackTimeoutMs);
      this.waiter = () => {
        if (!done()) return;
        clearTimeout(timer);
        this.waiter = undefined;
        resolve(true);
      };
    });
  }
}
