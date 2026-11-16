import type { Command, Telemetry } from "../protocol";

export type TransportState = "disconnected" | "connecting" | "connected";

/** The port the UI talks to. Implemented by BleTransport (real) and MockTransport (no hardware). */
export interface Transport {
  connect(): Promise<void>;
  disconnect(): void;
  send(cmd: Command): Promise<void>;
  onTelemetry(cb: (t: Telemetry) => void): void;
  onState(cb: (s: TransportState, info?: string) => void): void;
  /** Telemetry packets lost on the link, detected from sequence gaps. */
  readonly lost: number;
}
