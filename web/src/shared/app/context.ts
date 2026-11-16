import type { Telemetry } from "../protocol";
import type { Transport, TransportState } from "../transport";
import type { AppContext, TelemetryKind } from "./types";

/** Fan one Transport's single callbacks out to many feature subscribers. */
export function createContext(transport: Transport): AppContext {
  const telemetrySubs: Array<[TelemetryKind, (m: never) => void]> = [];
  const stateSubs: Array<(s: TransportState) => void> = [];
  const logSubs: Array<(m: string) => void> = [];
  const log = (m: string) => logSubs.forEach((f) => f(m));

  transport.onTelemetry((m) =>
    telemetrySubs.forEach(([kind, f]) => kind === m.type && (f as (m: Telemetry) => void)(m)),
  );
  transport.onState((s, info) => {
    stateSubs.forEach((f) => f(s));
    if (info) log(info);
  });

  return {
    transport,
    onTelemetry: (kind, cb) => telemetrySubs.push([kind, cb as (m: never) => void]),
    onState: (cb) => stateSubs.push(cb),
    log,
    onLog: (cb) => logSubs.push(cb),
  };
}
