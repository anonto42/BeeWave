import { $, panel, type AppContext } from "../../shared/app";
import type { TransportState } from "../../shared/transport";
import { statusNote } from "./note";
import { statusHtml } from "./template";

const KEYS = ["channel", "heap", "cpu", "dropped", "lost"];

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("STATUS", statusHtml);
  root.append(el);
  const set = (key: string, value: string) => ($(el, `[data-${key}]`).textContent = value);

  let link: TransportState = "disconnected";
  let got = false;
  const note = () => {
    const n = $(el, "[data-note]");
    n.textContent = statusNote(link, ctx.auth.state, got);
    n.hidden = n.textContent === "";
  };

  ctx.onTelemetry("status", (m) => {
    got = true;
    set("channel", m.activeChannel === 0 ? "AUTO" : String(m.activeChannel));
    set("heap", `${(m.freeHeap / 1024).toFixed(0)}K`);
    set("cpu", String(m.cpuLoad));
    set("dropped", String(m.dropped));
    set("lost", String(ctx.transport.lost));
    note();
  });
  ctx.onState((s) => {
    link = s;
    if (s === "disconnected") {
      got = false;
      for (const k of KEYS) set(k, "-");
    }
    note();
  });
  // the most recent alert (deauth, BLE, anomaly or watchlist) stays visible here
  ctx.monitor.alerts.onAlert((a) => {
    const last = $(el, "[data-last]");
    last.hidden = false;
    last.textContent = `last event: ${a.title}`;
  });
  ctx.auth.onChange(note);
  note();
}
