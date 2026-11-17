import { Authenticator, KeyStore } from "../auth";
import { createMonitorKernel } from "../monitor";
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
  // Every command goes through here, so the Log tab shows what was sent. Names only: never keys or payloads.
  const logged: Transport = {
    connect: () => transport.connect(),
    disconnect: () => transport.disconnect(),
    send: (c) => {
      if (c.type !== "otaChunk") log(`> ${c.type}`);
      return transport.send(c);
    },
    onTelemetry: (cb) => transport.onTelemetry(cb),
    onState: (cb) => transport.onState(cb),
    get lost() {
      return transport.lost;
    },
  };
  const auth = new Authenticator((c) => logged.send(c), keys);
  const monitor = createMonitorKernel();
  logSubs.push((m) => monitor.session.addLog(m));
  auth.onChange((s) => {
    monitor.session.addAuth(s);
    // the monitors' settings live on the device: read them as soon as we are allowed to
    if (s === "authenticated") void logged.send({ type: "getMonitor" }).catch(() => {});
  });

  transport.onTelemetry((m) => {
    monitor.ingest(m);
    void auth.handle(m);
    telemetrySubs.forEach(([kind, f]) => kind === m.type && (f as (m: Telemetry) => void)(m));
  });
  transport.onState((s, info) => {
    stateSubs.forEach((f) => f(s));
    if (s === "connected") void auth.start();
    if (s === "disconnected") auth.reset();
    if (info) log(info);
  });

  return {
    transport: logged,
    auth,
    keys,
    targets: new TargetStore(),
    monitor,
    onTelemetry: (kind, cb) => telemetrySubs.push([kind, cb as (m: never) => void]),
    onState: (cb) => stateSubs.push(cb),
    log,
    onLog: (cb) => logSubs.push(cb),
  };
}
