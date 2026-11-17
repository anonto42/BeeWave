import { RSSI_DELTA_DB } from "./constants";
import type { NewAlert } from "./alert-store";
import type { ScanInput } from "./scan-store";

const STORAGE_KEY = "beewave.watchlist";
/** A pinned network not reported for more than this many cycles counts as gone. */
export const MISSING_CYCLES = 5;

export interface WatchEntry {
  bssid: string;
  ssid: string;
}

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
    const pin = this.pinned(s.bssid);
    if (!pin) return [];
    const prev = this.state.get(s.bssid);
    const out: NewAlert[] = [];
    if (!prev?.present) {
      out.push(alert("info", "Watched network appeared", pin, `${s.rssi} dBm on channel ${s.channel}`));
    } else if (Math.abs(s.rssi - prev.rssi) >= RSSI_DELTA_DB) {
      const d = s.rssi - prev.rssi;
      out.push(alert("info", "Watched network moved", pin, `${d > 0 ? "+" : ""}${d} dB, now ${s.rssi} dBm`));
    }
    this.state.set(s.bssid, { present: true, lastCycle: cycle, rssi: s.rssi });
    return out;
  }

  /** Call once per cycle while scanning. */
  sweep(cycle: number): NewAlert[] {
    const out: NewAlert[] = [];
    for (const pin of this.all()) {
      const st = this.state.get(pin.bssid);
      if (st?.present && cycle - st.lastCycle > MISSING_CYCLES) {
        st.present = false;
        out.push(alert("warn", "Watched network disappeared", pin, `not heard for ${cycle - st.lastCycle} cycles`));
      }
    }
    return out;
  }
}

const alert = (severity: NewAlert["severity"], title: string, pin: WatchEntry, detail: string): NewAlert => ({
  source: "watchlist",
  severity,
  title,
  detail: `${pin.ssid || pin.bssid}: ${detail}`,
  bssid: pin.bssid,
});
