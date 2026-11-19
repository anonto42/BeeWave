import { CYCLE_TICKS, EVENT_CAP, HISTORY_CAP, SNAPSHOT_CAP } from "./constants";
import type { WifiPmfMode, WifiSecurityProtocol } from "../protocol";
import type { NetworkRecord, ScanEvent, SecurityRecord, Snapshot, WpsRecord } from "./types";

export interface ScanInput {
  bssid: string;
  ssid: string;
  channel: number;
  rssi: number;
}

/**
 * Everything the phone has heard in this session: one record per BSSID with its RSSI history, a list
 * of reports, and saved scans. A *cycle* is `CYCLE_TICKS` status heartbeats while the device is scanning.
 * Because the device only re-reports a known network when its RSSI moved by 6 dB or more, every
 * history point after the first is such a move: the history IS the de-duplicated stream.
 */
export class ScanStore {
  private records = new Map<string, NetworkRecord>();
  private log: ScanEvent[] = [];
  private shots: Snapshot[] = [];
  private subs: Array<() => void> = [];
  private ticks = 0;
  private nextShot = 1;
  /** When the current scan started; a snapshot holds only networks heard since then. */
  private scanFrom: number | null = null;

  constructor(private readonly now: () => number = Date.now) {}

  /** The current scan cycle number (0 until the first scanning heartbeats). */
  get cycle(): number {
    return Math.floor(this.ticks / CYCLE_TICKS);
  }

  onChange(cb: () => void): void {
    this.subs.push(cb);
  }

  networks(): NetworkRecord[] {
    return [...this.records.values()];
  }

  get(bssid: string): NetworkRecord | undefined {
    return this.records.get(bssid);
  }

  /** Every report received, oldest first (capped). */
  events(): readonly ScanEvent[] {
    return this.log;
  }

  ingest(m: ScanInput): void {
    const at = this.now();
    const cycle = this.cycle;
    const rec = this.records.get(m.bssid);
    if (rec) {
      if (!rec.ssid && m.ssid) {
        rec.decloaked = true;
      }
      rec.ssid = m.ssid || rec.ssid;
      rec.channel = m.channel;
      rec.rssi = m.rssi;
      rec.lastSeen = at;
      rec.count++;
      rec.history.push({ cycle, rssi: m.rssi, at });
      if (rec.history.length > HISTORY_CAP) rec.history.shift();
    } else {
      this.records.set(m.bssid, {
        bssid: m.bssid,
        ssid: m.ssid,
        channel: m.channel,
        rssi: m.rssi,
        firstSeen: at,
        lastSeen: at,
        count: 1,
        history: [{ cycle, rssi: m.rssi, at }],
      });
    }
    this.log.push({ at, bssid: m.bssid, ssid: m.ssid, channel: m.channel, rssi: m.rssi });
    if (this.log.length > EVENT_CAP) this.log.shift();
    this.emit();
  }

  ingestDecloaked(m: { bssid: string; ssid: string; channel: number; rssi: number }): void {
    const at = this.now();
    const cycle = this.cycle;
    const rec = this.records.get(m.bssid);
    if (rec) {
      rec.ssid = m.ssid;
      rec.channel = m.channel;
      rec.rssi = m.rssi;
      rec.lastSeen = at;
      rec.count++;
      rec.decloaked = true;
      rec.history.push({ cycle, rssi: m.rssi, at });
      if (rec.history.length > HISTORY_CAP) rec.history.shift();
    } else {
      this.records.set(m.bssid, {
        bssid: m.bssid,
        ssid: m.ssid,
        channel: m.channel,
        rssi: m.rssi,
        firstSeen: at,
        lastSeen: at,
        count: 1,
        decloaked: true,
        history: [{ cycle, rssi: m.rssi, at }],
      });
    }
    this.log.push({ at, bssid: m.bssid, ssid: m.ssid, channel: m.channel, rssi: m.rssi });
    if (this.log.length > EVENT_CAP) this.log.shift();
    this.emit();
  }

  ingestWps(m: {
    bssid: string;
    version: number;
    state: number;
    locked: boolean;
    selectedRegistrar: boolean;
    deviceName: string;
    manufacturer: string;
    modelName: string;
  }): void {
    const rec = this.records.get(m.bssid);
    const wps: WpsRecord = {
      version: m.version,
      state: m.state,
      locked: m.locked,
      selectedRegistrar: m.selectedRegistrar,
      deviceName: m.deviceName,
      manufacturer: m.manufacturer,
      modelName: m.modelName,
    };
    if (rec) {
      rec.wps = wps;
      this.emit();
    } else {
      const at = this.now();
      const cycle = this.cycle;
      this.records.set(m.bssid, {
        bssid: m.bssid,
        ssid: m.deviceName,
        channel: 1,
        rssi: -90,
        firstSeen: at,
        lastSeen: at,
        count: 1,
        history: [{ cycle, rssi: -90, at }],
        wps,
      });
      this.emit();
    }
  }

  ingestSecurity(m: {
    bssid: string;
    protocol: WifiSecurityProtocol;
    pmf: WifiPmfMode;
  }): void {
    const rec = this.records.get(m.bssid);
    const security: SecurityRecord = {
      protocol: m.protocol,
      pmf: m.pmf,
    };
    if (rec) {
      rec.security = security;
      this.emit();
    } else {
      const at = this.now();
      const cycle = this.cycle;
      this.records.set(m.bssid, {
        bssid: m.bssid,
        ssid: "",
        channel: 1,
        rssi: -90,
        firstSeen: at,
        lastSeen: at,
        count: 1,
        history: [{ cycle, rssi: -90, at }],
        security,
      });
      this.emit();
    }
  }

  /** Call once per status heartbeat while the device reports `scanning`; advances the cycle counter. */
  pulse(scanning: boolean): void {
    if (!scanning) return;
    this.ticks++;
    if (this.ticks % CYCLE_TICKS === 0) this.emit(); // a new cycle: views that depend on "now" refresh
  }

  /** A new scan started: from now on a snapshot holds only what this scan hears. */
  beginScan(): void {
    this.scanFrom = this.now();
  }

  /**
   * Save what the current scan heard (or everything, if no scan was started) for later comparison. The device
   * clears its table when a scan starts, so each scan re-reports every network it hears.
   */
  snapshot(label: string): Snapshot {
    const from = this.scanFrom;
    const heard = this.networks().filter((n) => from === null || n.lastSeen >= from);
    const shot: Snapshot = {
      id: this.nextShot++,
      label,
      at: this.now(),
      networks: heard.map(({ bssid, ssid, channel, rssi }) => ({ bssid, ssid, channel, rssi })),
    };
    this.shots.push(shot);
    if (this.shots.length > SNAPSHOT_CAP) this.shots.shift();
    this.emit();
    return shot;
  }

  snapshots(): readonly Snapshot[] {
    return this.shots;
  }

  clear(): void {
    this.records.clear();
    this.log = [];
    this.emit();
  }

  private emit(): void {
    this.subs.forEach((f) => f());
  }
}
