import { Authenticator, KeyStore } from "../auth";
import { createMonitorKernel } from "../monitor";
import { createSettingsKernel } from "../settings";
import type { Telemetry } from "../protocol";
import { TargetStore } from "../targets";
import type { Transport, TransportState } from "../transport";
import type { AppContext, TelemetryKind } from "./types";

/** How long to wait for `SettingsAll` before deciding the firmware does not support settings. */
const SETTINGS_REPLY_MS = 4000;
const RECONNECT_TRIES = 3;
const RECONNECT_DELAY_MS = 2000;

/** Try to get the link back after an unexpected drop: a few attempts, spaced out, then give up (the user can tap Connect). */
async function reconnect(connect: () => Promise<void>, log: (m: string) => void): Promise<void> {
  for (let i = 1; i <= RECONNECT_TRIES; i++) {
    await new Promise((r) => setTimeout(r, RECONNECT_DELAY_MS * i));
    log(`auto-reconnect: attempt ${i} of ${RECONNECT_TRIES}`);
    try {
      await connect();
      return;
    } catch {
      /* try again */
    }
  }
  log("auto-reconnect gave up; tap Connect");
}

/** Fan one Transport's single callbacks out to many feature subscribers. */
export function createContext(transport: Transport): AppContext {
  const telemetrySubs: Array<[TelemetryKind, (m: never) => void]> = [];
  const stateSubs: Array<(s: TransportState) => void> = [];
  const logSubs: Array<(m: string) => void> = [];
  const log = (m: string) => logSubs.forEach((f) => f(m));

  const keys = new KeyStore();
  // Every command goes through here, so the Log tab shows what was sent. Names only: never keys or payloads.
  /** True while the user is the one closing the link (no auto-reconnect then). */
  let userClosed = false;
  let reconnecting = false;
  const logged: Transport = {
    connect: () => {
      userClosed = false;
      return transport.connect();
    },
    disconnect: () => {
      userClosed = true;
      transport.disconnect();
    },
    send: (c) => {
      if (c.type !== "otaChunk") log(`> ${c.type}`);
      return transport.send(c);
    },
    onTelemetry: (cb) => transport.onTelemetry(cb),
    onState: (cb) => transport.onState(cb),
    get lost() {
      return transport.lost;
    },
    get deviceName() {
      return transport.deviceName ?? null;
    },
  };
  const auth = new Authenticator((c) => logged.send(c), keys);
  const monitor = createMonitorKernel();
  const settings = createSettingsKernel();
  keys.setPersist(settings.phone.get().session.rememberKey);
  settings.phone.onChange((s) => keys.setPersist(s.session.rememberKey));
  logSubs.push((m) => monitor.session.addLog(m));
  // alerts (device and watchlist) also appear in the Log tab
  monitor.alerts.onAlert((a) => log(`alert [${a.source}]: ${a.title}${a.detail ? ` - ${a.detail}` : ""}`));
  auth.onChange((s) => {
    monitor.session.addAuth(s);
    // the monitors' settings live on the device: read them as soon as we are allowed to
    // Device settings are requested only after AuthResult ok: before that the device would refuse anyway.
    if (s === "authenticated") {
      for (const type of ["getSettingsAll", "getDeviceInfo", "getMonitor", "getHandshakeCapture"] as const) void logged.send({ type }).catch(() => {});
      // firmware without settings support never answers: say so instead of waiting forever
      setTimeout(() => settings.device.markUnsupported(), SETTINGS_REPLY_MS);
    }
  });

  transport.onTelemetry((m) => {
    monitor.ingest(m);
    settings.device.ingest(m);
    settings.state.ingest(m);
    void auth.handle(m);
    telemetrySubs.forEach(([kind, f]) => kind === m.type && (f as (m: Telemetry) => void)(m));
  });
  transport.onState((s, info) => {
    if (s === "connected" && transport.deviceName) {
      const name = transport.deviceName;
      if (settings.phone.get().session.lastDeviceName !== name) settings.phone.update((p) => void (p.session.lastDeviceName = name));
    }
    if (s === "disconnected") {
      settings.device.reset();
      settings.state.reset();
      // a failed attempt also reports "disconnected": only one reconnect loop at a time
      if (!userClosed && !reconnecting && settings.phone.get().session.autoReconnect && transport.reconnect) {
        reconnecting = true;
        void reconnect(transport.reconnect.bind(transport), log).finally(() => (reconnecting = false));
      }
    }
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
    settings,
    onTelemetry: (kind, cb) => telemetrySubs.push([kind, cb as (m: never) => void]),
    onState: (cb) => stateSubs.push(cb),
    log,
    onLog: (cb) => logSubs.push(cb),
  };
}
