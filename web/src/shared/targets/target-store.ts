import type { Target } from "./types";

const KEY = "beewave.targets";

/** The user's target networks, shared by the scanner and the tray. Persisted in the browser only. */
export class TargetStore {
  private items = new Map<string, Target>();
  private subs: Array<() => void> = [];

  constructor(private readonly storage: Pick<Storage, "getItem" | "setItem"> | null = safeStorage()) {
    try {
      const raw = storage?.getItem(KEY);
      for (const t of raw ? (JSON.parse(raw) as Target[]) : []) this.items.set(t.bssid, t);
    } catch {
      /* corrupted or blocked storage: start empty */
    }
  }

  list(): Target[] {
    return [...this.items.values()];
  }

  has(bssid: string): boolean {
    return this.items.has(bssid);
  }

  toggle(t: Target): void {
    if (this.items.has(t.bssid)) this.items.delete(t.bssid);
    else this.items.set(t.bssid, t);
    this.persist();
    this.subs.forEach((f) => f());
  }

  remove(bssid: string): void {
    if (this.items.delete(bssid)) {
      this.persist();
      this.subs.forEach((f) => f());
    }
  }

  onChange(cb: () => void): void {
    this.subs.push(cb);
  }

  private persist(): void {
    try {
      this.storage?.setItem(KEY, JSON.stringify(this.list()));
    } catch {
      /* private mode / quota: keep working in memory */
    }
  }
}

function safeStorage(): Storage | null {
  try {
    return typeof localStorage === "undefined" ? null : localStorage;
  } catch {
    return null;
  }
}
