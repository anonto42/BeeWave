import type { Telemetry } from "../protocol";
import type { Target, TargetStore } from "../targets";
import type { Transport, TransportState } from "../transport";

export type { Target };
export type TelemetryKind = Telemetry["type"];

/** What every feature is wired with. Features depend on this, never on each other. */
export interface AppContext {
  readonly transport: Transport;
  /** Networks the user marked as targets (shared by scanner and tray). */
  readonly targets: TargetStore;
  /** Subscribe to one kind of telemetry message. */
  onTelemetry<K extends TelemetryKind>(kind: K, cb: (m: Extract<Telemetry, { type: K }>) => void): void;
  onState(cb: (s: TransportState) => void): void;
  log(msg: string): void;
  onLog(cb: (msg: string) => void): void;
}
