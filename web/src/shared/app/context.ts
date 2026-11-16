import { Authenticator, KeyStore } from "../auth";
import type { Telemetry } from "../protocol";
import { TargetStore } from "../targets";
import type { Transport, TransportState } from "../transport";
import type { AppContext, TelemetryKind } from "./types";

/** Fan one Transport's single callbacks out to many feature subscribers. */
export function createContext(transport: Transport): AppContext {
  const telemetrySubs: Array<[TelemetryKind, (m: never) => void]> = [];
  const stateSubs: Array<(s: TransportState) => void> = [];
  const logSubs: Array<(m: string) => void> = [];
  const log = (m: string) => logSubs.forEach((f) => f(m));

  const keys = new KeyStore();
  const auth = new Authenticator((c) => transport.send(c), keys);

  transport.onTelemetry((m) => {
    void auth.handle(m);
    telemetrySubs.forEach(([kind, f]) => kind === m.type && (f as (m: Telemetry) => void)(m));
  });
  transport.onState((s, info) => {
    if (s === "connected") void auth.start();
    if (s === "disconnected") auth.reset();
    stateSubs.forEach((f) => f(s));
    if (info) log(info);
  });

  return {
    transport,
    auth,
    keys,
    targets: new TargetStore(),
    onTelemetry: (kind, cb) => telemetrySubs.push([kind, cb as (m: never) => void]),
    onState: (cb) => stateSubs.push(cb),
    log,
    onLog: (cb) => logSubs.push(cb),
  };
}
