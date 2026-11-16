import type { Telemetry } from "../protocol";
import type { Transport, TransportState } from "../transport";

export type TelemetryKind = Telemetry["type"];

/** What every feature is wired with. Features depend on this, never on each other. */
export interface AppContext {
  readonly transport: Transport;
  /** Subscribe to one kind of telemetry message. */
  onTelemetry<K extends TelemetryKind>(kind: K, cb: (m: Extract<Telemetry, { type: K }>) => void): void;
  onState(cb: (s: TransportState) => void): void;
  log(msg: string): void;
  onLog(cb: (msg: string) => void): void;
}
