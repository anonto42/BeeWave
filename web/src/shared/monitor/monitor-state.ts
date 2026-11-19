import type { AdClassName, Telemetry } from "../protocol";
import { AD_CLASSES } from "../protocol";

export const SERIES_CAP = 60;

export interface MonitorConfig {
  deauthThreshold: number;
  bleEnabled: boolean;
  bleSpikeFactor: number;
}

/** What the device uses until told otherwise (mirrors `MonitorConfig::default` in the core). */
export const DEFAULT_CONFIG: MonitorConfig = { deauthThreshold: 10, bleEnabled: false, bleSpikeFactor: 3 };

export interface DeauthSource {
  bssid: string;
  /** Frames per second in the latest alert. */
  last: number;
  alerts: number;
  lastAt: number;
  /** Frames per second over the alerts received (oldest first). */
  series: number[];
}

export type BleCounts = Record<AdClassName, number>;

/** What the receive-only monitors have reported: config, deauth sources and BLE counts over time. */
export class MonitorState {
  config: MonitorConfig | null = null;
  readonly deauth = new Map<string, DeauthSource>();
  bleLatest: BleCounts | null = null;
  readonly bleSeries: Record<AdClassName, number[]> = { fastPair: [], ibeacon: [], proximity: [], generic: [], findMy: [] };
  private subs: Array<() => void> = [];

  constructor(private readonly now: () => number = Date.now) {}

  onChange(cb: () => void): void {
    this.subs.push(cb);
  }

  /** The config to show: the device's, or the defaults until it has answered. */
  effective(): MonitorConfig {
    return this.config ?? DEFAULT_CONFIG;
  }

  ingest(m: Telemetry): void {
    if (m.type === "monitorConfig") {
      this.config = { deauthThreshold: m.deauthThreshold, bleEnabled: m.bleEnabled, bleSpikeFactor: m.bleSpikeFactor };
    } else if (m.type === "deauth" || m.type === "disassocAlert") {
      const s = this.deauth.get(m.bssid) ?? { bssid: m.bssid, last: 0, alerts: 0, lastAt: 0, series: [] };
      s.last = m.count;
      s.alerts++;
      s.lastAt = this.now();
      s.series.push(m.count);
      if (s.series.length > SERIES_CAP) s.series.shift();
      this.deauth.set(m.bssid, s);
    } else if (m.type === "bleStats") {
      this.bleLatest = { fastPair: m.fastPair, ibeacon: m.ibeacon, proximity: m.proximity, generic: m.generic, findMy: m.findMy };
      for (const c of AD_CLASSES) {
        const series = this.bleSeries[c];
        series.push(this.bleLatest[c]);
        if (series.length > SERIES_CAP) series.shift();
      }
    } else return;
    this.subs.forEach((f) => f());
  }

  /** Forget BLE history (the monitor was switched off or on: old counts no longer mean anything). */
  resetBle(): void {
    this.bleLatest = null;
    for (const c of AD_CLASSES) this.bleSeries[c] = [];
    this.subs.forEach((f) => f());
  }
}
