import { $, panel, type AppContext } from "../../shared/app";
import { statusHtml } from "./template";

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("STATUS", statusHtml);
  root.append(el);
  const set = (key: string, value: string) => ($(el, `[data-${key}]`).textContent = value);

  ctx.onTelemetry("status", (m) => {
    set("channel", String(m.activeChannel));
    set("heap", `${(m.freeHeap / 1024).toFixed(0)}K`);
    set("cpu", String(m.cpuLoad));
    set("dropped", String(m.dropped));
    set("lost", String(ctx.transport.lost));
  });
}
