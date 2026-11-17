import type { AlertRecord, ScanEvent, Snapshot } from "./types";

export interface SessionData {
  exportedAt: string;
  startedAt: string;
  scans: ScanEvent[];
  alerts: AlertRecord[];
  log: Array<{ at: number; text: string }>;
  auth: Array<{ at: number; state: string }>;
  snapshots: Snapshot[];
}

/** Log lines and authentication events, collected as they happen. */
export class SessionRecorder {
  readonly startedAt: number;
  private logLines: Array<{ at: number; text: string }> = [];
  private authEvents: Array<{ at: number; state: string }> = [];

  constructor(private readonly now: () => number = Date.now) {
    this.startedAt = now();
  }

  addLog(text: string): void {
    this.logLines.push({ at: this.now(), text });
    if (this.logLines.length > 2000) this.logLines.shift();
  }

  addAuth(state: string): void {
    this.authEvents.push({ at: this.now(), state });
  }

  build(scans: readonly ScanEvent[], alerts: readonly AlertRecord[], snapshots: readonly Snapshot[]): SessionData {
    return {
      exportedAt: new Date(this.now()).toISOString(),
      startedAt: new Date(this.startedAt).toISOString(),
      scans: [...scans],
      alerts: [...alerts].reverse(), // oldest first, like everything else in the file
      log: [...this.logLines],
      auth: [...this.authEvents],
      snapshots: [...snapshots],
    };
  }
}

export const sessionToJson = (d: SessionData): string => JSON.stringify(d, null, 2);

const CSV_HEADER = ["kind", "time", "bssid", "ssid", "channel", "rssi", "severity", "source", "message"];

/**
 * A text cell: quoted when needed, and prefixed with an apostrophe if a spreadsheet would read it as a formula.
 * SSIDs and device messages are attacker-controlled text, so `=HYPERLINK(...)` must not survive as a formula.
 */
export function csvText(value: string): string {
  const safe = /^[=+\-@\t\r]/.test(value) ? `'${value}` : value;
  return /[",\r\n]/.test(safe) ? `"${safe.replace(/"/g, '""')}"` : safe;
}

const iso = (ms: number) => new Date(ms).toISOString();

/** One flat CSV: a `kind` column says whether a row is a scan report, an alert, a log line or an auth event. */
export function sessionToCsv(d: SessionData): string {
  const rows: string[] = [CSV_HEADER.join(",")];
  const row = (cells: Array<string | number>) => rows.push(cells.map((c) => (typeof c === "number" ? String(c) : csvText(c))).join(","));
  for (const s of d.scans) row(["scan", iso(s.at), s.bssid, s.ssid, s.channel, s.rssi, "", "", ""]);
  for (const a of d.alerts) row(["alert", iso(a.at), a.bssid ?? "", "", "", "", a.severity, a.source, `${a.title}: ${a.detail}`]);
  for (const l of d.log) row(["log", iso(l.at), "", "", "", "", "", "", l.text]);
  for (const e of d.auth) row(["auth", iso(e.at), "", "", "", "", "", "", e.state]);
  return rows.join("\r\n") + "\r\n";
}
