import type { GpsCoordinates } from "../gps";
import type { Telemetry } from "../protocol";
import { AlertStore } from "./alert-store";
import { AssocStore } from "./assoc-store";
import { HandshakeState } from "./handshake-store";
import { alertFromTelemetry } from "./alerts";
import { MonitorState } from "./monitor-state";
import { ProbeStore } from "./probe-store";
import { ScanStore } from "./scan-store";
import { SessionRecorder } from "./session";
import { WatchlistEngine, WatchlistStore } from "./watchlist";

/** Every store the monitoring panels read, fed from one place so a panel in a hidden tab loses nothing. */
export interface MonitorKernel {
  readonly scans: ScanStore;
  readonly alerts: AlertStore;
  readonly watchlist: WatchlistStore;
  readonly state: MonitorState;
  readonly handshake: HandshakeState;
  readonly probes: ProbeStore;
  readonly assoc: AssocStore;
  readonly session: SessionRecorder;
  /** Feed every telemetry message here. */
  ingest(m: Telemetry): void;
}

export function createMonitorKernel(
  now: () => number = Date.now,
  watchStorage?: ConstructorParameters<typeof WatchlistStore>[0],
  getGps: () => GpsCoordinates | null = () => null
): MonitorKernel {
  const scans = new ScanStore(now, getGps);
  const alerts = new AlertStore(now);
  const watchlist = new WatchlistStore(watchStorage);
  const state = new MonitorState(now);
  const handshake = new HandshakeState(now);
  const probes = new ProbeStore(now);
  const assoc = new AssocStore(now);
  const session = new SessionRecorder(now);
  const engine = new WatchlistEngine(
    (bssid) => watchlist.list().find((e) => e.bssid === bssid),
    () => watchlist.list(),
  );

  let scanning = false;
  const raise = (list: ReturnType<WatchlistEngine["sweep"]>) => list.forEach((a) => alerts.add(a));

  return {
    scans,
    alerts,
    watchlist,
    state,
    handshake,
    probes,
    assoc,
    session,
    ingest(m) {
      switch (m.type) {
        case "probeResult":
          probes.ingest(m);
          return;
        case "assocResult":
          assoc.ingest(m);
          return;
        case "wpsResult":
          scans.ingestWps(m);
          return;
        case "securityResult":
          scans.ingestSecurity(m);
          return;
        case "phyResult":
          scans.ingestPhy(m);
          return;
        case "decloakedResult": {
          scans.ingestDecloaked(m);
          const a = alertFromTelemetry(m);
          if (a) alerts.add(a);
          return;
        }
        case "scan":
          scans.ingest(m);
          raise(engine.observe(m, scans.cycle));
          return;
        case "status":
          if (m.scanning && !scanning) scans.beginScan();
          scans.pulse(m.scanning);
          if (m.scanning) raise(engine.sweep(scans.cycle));
          // a finished scan is worth keeping for the comparison
          if (scanning && !m.scanning && scans.networks().length > 0) {
            scans.snapshot(`Scan ${new Date(now()).toLocaleTimeString()}`);
          }
          scanning = m.scanning;
          return;
        case "monitorConfig": {
          const wasOn = state.effective().bleEnabled;
          state.ingest(m);
          if (wasOn !== m.bleEnabled) state.resetBle();
          return;
        }
        case "bleStats":
          state.ingest(m);
          return;
        case "handshakeCaptureState":
        case "handshakeFrame":
        case "handshakeComplete":
          handshake.ingest(m);
          if (m.type === "handshakeComplete") {
            const a = alertFromTelemetry(m);
            if (a) alerts.add(a);
          }
          return;
        default: {
          state.ingest(m);
          const a = alertFromTelemetry(m);
          if (a) alerts.add(a);
        }
      }
    },
  };
}
