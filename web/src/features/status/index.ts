import { $, panel, type AppContext } from "../../shared/app";

export function mount(root: HTMLElement, ctx: AppContext) {
  const cell = (k: string, label: string) => `<div><b data-${k}>-</b><span>${label}</span></div>`;
  const el = panel("STATUS", `<div class="stat">${cell("ch", "CHANNEL")}${cell("heap", "FREE HEAP")}${cell("cpu", "CPU %")}${cell("drop", "DEV DROPPED")}${cell("lost", "BLE LOST")}</div>`);
  root.append(el);
  const set = (k: string, v: string) => ($(el, `[data-${k}]`).textContent = v);
  ctx.onTelemetry("status", (m) => {
    set("ch", String(m.activeChannel));
    set("heap", `${(m.freeHeap / 1024).toFixed(0)}K`);
    set("cpu", String(m.cpuLoad));
    set("drop", String(m.dropped));
    set("lost", String(ctx.transport.lost));
  });
}
