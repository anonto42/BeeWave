import { ACTIVITY_WINDOW_MS, RSSI_DELTA_DB } from "./constants";
import type { NetworkRecord, ScanEvent, Snapshot, SnapshotNetwork } from "./types";

export const CHANNELS = Array.from({ length: 13 }, (_, i) => i + 1);

/** Access points per channel 1-13 (index 0 = channel 1). */
export function channelOccupancy(networks: readonly Pick<NetworkRecord, "channel">[]): number[] {
  const out = CHANNELS.map(() => 0);
  for (const n of networks) if (n.channel >= 1 && n.channel <= 13) out[n.channel - 1]++;
  return out;
}

/**
 * Reports per channel over the last minute (index 0 = channel 1). It is a proxy for how busy a channel
 * is, NOT airtime utilisation: the device only sees beacons, and only reports changes.
 */
export function channelActivity(events: readonly ScanEvent[], now: number, windowMs = ACTIVITY_WINDOW_MS): number[] {
  const out = CHANNELS.map(() => 0);
  for (let i = events.length - 1; i >= 0; i--) {
    const e = events[i];
    if (now - e.at > windowMs) break; // events are oldest first
    if (e.channel >= 1 && e.channel <= 13) out[e.channel - 1]++;
  }
  return out;
}

/** RSSI change between the last two reports of a network, or null if it has only been reported once. */
export function lastDelta(rec: Pick<NetworkRecord, "history">): number | null {
  const h = rec.history;
  return h.length < 2 ? null : h[h.length - 1].rssi - h[h.length - 2].rssi;
}

export interface DeltaRow {
  record: NetworkRecord;
  delta: number;
}

/**
 * The "delta view": networks whose RSSI moved by at least `minDb` in the latest cycle (or the one before).
 * The device de-duplicates, so a re-report IS such a move: this makes that rule visible. Biggest move first.
 */
export function deltaView(records: readonly NetworkRecord[], currentCycle: number, minDb = RSSI_DELTA_DB): DeltaRow[] {
  const rows: DeltaRow[] = [];
  for (const record of records) {
    const delta = lastDelta(record);
    const last = record.history[record.history.length - 1];
    if (delta === null || !last || currentCycle - last.cycle > 1) continue;
    if (Math.abs(delta) >= minDb) rows.push({ record, delta });
  }
  return rows.sort((a, b) => Math.abs(b.delta) - Math.abs(a.delta));
}

/** SVG polyline points for a sparkline of `values` in a `w` x `h` box (a flat line for a constant series). */
export function sparklinePoints(values: readonly number[], w: number, h: number): string {
  if (values.length === 0) return "";
  const lo = Math.min(...values);
  const hi = Math.max(...values);
  const span = hi - lo || 1;
  const step = values.length > 1 ? w / (values.length - 1) : 0;
  return values
    .map((v, i) => `${(i * step).toFixed(1)},${(h - ((v - lo) / span) * (h - 2) - 1).toFixed(1)}`)
    .join(" ");
}

export interface SnapshotDiff {
  added: SnapshotNetwork[];
  removed: SnapshotNetwork[];
  rssiChanged: Array<{ before: SnapshotNetwork; after: SnapshotNetwork; delta: number }>;
  channelChanged: Array<{ before: SnapshotNetwork; after: SnapshotNetwork }>;
}

/** What changed from scan `a` to scan `b`. RSSI counts as changed at `minDb` dB or more (the dedup threshold). */
export function diffSnapshots(a: Pick<Snapshot, "networks">, b: Pick<Snapshot, "networks">, minDb = RSSI_DELTA_DB): SnapshotDiff {
  const before = new Map(a.networks.map((n) => [n.bssid, n]));
  const after = new Map(b.networks.map((n) => [n.bssid, n]));
  const diff: SnapshotDiff = { added: [], removed: [], rssiChanged: [], channelChanged: [] };
  for (const [bssid, n] of after) {
    const old = before.get(bssid);
    if (!old) diff.added.push(n);
    else {
      if (old.channel !== n.channel) diff.channelChanged.push({ before: old, after: n });
      const delta = n.rssi - old.rssi;
      if (Math.abs(delta) >= minDb) diff.rssiChanged.push({ before: old, after: n, delta });
    }
  }
  for (const [bssid, n] of before) if (!after.has(bssid)) diff.removed.push(n);
  return diff;
}
