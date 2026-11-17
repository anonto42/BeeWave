import { ALERT_CAP } from "./constants";
import type { AlertRecord } from "./types";

export type NewAlert = Omit<AlertRecord, "id" | "at">;

/** Alerts raised during the session (device alerts and watchlist hits), newest first. */
export class AlertStore {
  private items: AlertRecord[] = [];
  private subs: Array<(a: AlertRecord) => void> = [];
  private changeSubs: Array<() => void> = [];
  private next = 1;

  constructor(private readonly now: () => number = Date.now) {}

  /** Called for every new alert (the session log and the UI badge use it). */
  onAlert(cb: (a: AlertRecord) => void): void {
    this.subs.push(cb);
  }

  /** Called whenever the list changes (a new alert, or cleared). */
  onChange(cb: () => void): void {
    this.changeSubs.push(cb);
  }

  add(a: NewAlert): AlertRecord {
    const rec: AlertRecord = { ...a, id: this.next++, at: this.now() };
    this.items.unshift(rec);
    if (this.items.length > ALERT_CAP) this.items.pop();
    this.subs.forEach((f) => f(rec));
    this.changeSubs.forEach((f) => f());
    return rec;
  }

  list(): readonly AlertRecord[] {
    return this.items;
  }

  clear(): void {
    this.items = [];
    this.changeSubs.forEach((f) => f());
  }
}
