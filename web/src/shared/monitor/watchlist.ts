import { RSSI_DELTA_DB } from "./constants";
import type { NewAlert } from "./alert-store";
import type { ScanInput } from "./scan-store";

const STORAGE_KEY = "beewave.watchlist";
/** A pinned network not reported for more than this many cycles counts as gone. */
export const MISSING_CYCLES = 5;

/** A pinned network and which events matter for it. Older saved entries (just bssid and ssid) get the defaults. */
export interface WatchEntry {
  bssid: string;
  ssid: string;
  label?: string;
  /** Alert when it appears (default on). */
  appear?: boolean;
  /** Alert when it disappears (default on). */
  disappear?: boolean;
  /** Alert when RSSI moves by this many dB or more; 0 = off (default 6). */
  deltaDb?: number;
  enabled?: boolean;
}

/** An entry with every optional field filled in. */
export const fullEntry = (e: WatchEntry): Required<WatchEntry> => ({
  bssid: e.bssid,
  ssid: e.ssid,
  label: e.label ?? "",
  appear: e.appear ?? true,
  disappear: e.disappear ?? true,
  deltaDb: e.deltaDb ?? RSSI_DELTA_DB,
  enabled: e.enabled ?? true,
});

/** Pinned BSSIDs, kept in this browser only (the device never sees them). Works in memory if storage is blocked. */
export class WatchlistStore {
  private entries: WatchEntry[] = [];
  private subs: Array<() => void> = [];

  constructor(private readonly storage: Pick<Storage, "getItem" | "setItem"> | null = safeStorage()) {
    try {
      const raw = storage?.getItem(STORAGE_KEY);
      const parsed: unknown = raw ? JSON.parse(raw) : [];
      if (Array.isArray(parsed)) {
        this.entries = parsed.filter((e): e is WatchEntry => typeof e?.bssid === "string" && typeof e?.ssid === "string");
      }
    } catch {
      /* blocked or corrupt storage: start empty */
    }
  }

  list(): readonly WatchEntry[] {
    return this.entries;
  }

  has(bssid: string): boolean {
    return this.entries.some((e) => e.bssid === bssid);
  }

  add(e: WatchEntry): void {
    if (!this.has(e.bssid)) this.entries.push(e);
    this.save();
  }

  /** Change a pinned entry (label, rules, enabled). */
  update(bssid: string, patch: Partial<Omit<WatchEntry, "bssid">>): void {
    this.entries = this.entries.map((e) => (e.bssid === bssid ? { ...e, ...patch } : e));
    this.save();
  }

  /** Replace everything (an import). */
  replaceAll(entries: readonly WatchEntry[]): void {
    this.entries = [...entries];
    this.save();
  }

  remove(bssid: string): void {
    this.entries = this.entries.filter((e) => e.bssid !== bssid);
    this.save();
  }

  onChange(cb: () => void): void {
    this.subs.push(cb);
  }

  private save(): void {
    try {
      this.storage?.setItem(STORAGE_KEY, JSON.stringify(this.entries));
    } catch {
      /* private mode / quota: keep working in memory */
    }
    this.subs.forEach((f) => f());
  }
}

function safeStorage(): Storage | null {
  try {
    return typeof localStorage === "undefined" ? null : localStorage;
  } catch {
    return null;
  }
}

interface WatchState {
  present: boolean;
  lastCycle: number;
  rssi: number;
}

/**
 * Raises alerts for pinned BSSIDs: one appears, one has been missing for more than `MISSING_CYCLES`
 * cycles, or one moves by `RSSI_DELTA_DB` or more between reports. Pure rules over the scan stream.
 */
export class WatchlistEngine {
  private state = new Map<string, WatchState>();

  constructor(private readonly pinned: (bssid: string) => WatchEntry | undefined, private readonly all: () => readonly WatchEntry[]) {}

  observe(s: ScanInput, cycle: number): NewAlert[] {
    const found = this.pinned(s.bssid);
    if (!found) return [];
    const pin = fullEntry(found);
    const prev = this.state.get(s.bssid);
    const out: NewAlert[] = [];
    // state is tracked even while the entry is off, so switching it on does not fire for old news
    if (pin.enabled) {
      if (!prev?.present) {
        if (pin.appear) out.push(alert("info", "Watched network appeared", pin, `${s.rssi} dBm on channel ${s.channel}`));
      } else if (pin.deltaDb > 0 && Math.abs(s.rssi - prev.rssi) >= pin.deltaDb) {
        const d = s.rssi - prev.rssi;
        out.push(alert("info", "Watched network moved", pin, `${d > 0 ? "+" : ""}${d} dB, now ${s.rssi} dBm`));
      }
    }
    this.state.set(s.bssid, { present: true, lastCycle: cycle, rssi: s.rssi });
    return out;
  }

  /** Call once per cycle while scanning. */
  sweep(cycle: number): NewAlert[] {
    const out: NewAlert[] = [];
    for (const e of this.all()) {
      const pin = fullEntry(e);
      const st = this.state.get(pin.bssid);
      if (st?.present && cycle - st.lastCycle > MISSING_CYCLES) {
        st.present = false;
        if (pin.enabled && pin.disappear) out.push(alert("warn", "Watched network disappeared", pin, `not heard for ${cycle - st.lastCycle} cycles`));
      }
    }
    return out;
  }
}

const alert = (severity: NewAlert["severity"], title: string, pin: Required<WatchEntry>, detail: string): NewAlert => ({
  source: "watchlist",
  severity,
  title,
  detail: `${pin.label || pin.ssid || pin.bssid}: ${detail}`,
  bssid: pin.bssid,
});
