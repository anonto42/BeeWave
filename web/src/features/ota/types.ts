export interface OtaOptions {
  /** Chunks allowed in flight before waiting for acks. */
  window: number;
  /** Bytes per chunk (device limit 96). */
  chunkSize: number;
  /** Rewind and resend if no ack arrives within this time. */
  ackTimeoutMs: number;
  /** Give up after this many consecutive timeouts. */
  maxRetries: number;
}

export type OtaProgress = (sent: number, total: number) => void;
