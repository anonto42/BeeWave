import { $, panel, type AppContext } from "../../shared/app";
import { rowHtml, scannerHtml } from "./template";
import type { ScanResult } from "./types";

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("SCANNER", scannerHtml);
  root.append(el);
  const body = $(el, "tbody");
  const networks = new Map<string, ScanResult>();

  const render = () => {
    body.innerHTML = [...networks.values()]
      .sort((a, b) => b.rssi - a.rssi)
      .map((n) => rowHtml(n, ctx.targets.has(n.bssid)))
      .join("");
  };

  body.onclick = (e) => {
    const row = (e.target as HTMLElement).closest<HTMLElement>("tr[data-bssid]");
    if (row) ctx.targets.toggle({ bssid: row.dataset.bssid!, ssid: row.dataset.ssid ?? "" });
  };

  ctx.targets.onChange(render);
  ctx.onTelemetry("scan", (m) => {
    networks.set(m.bssid, m);
    render();
  });
}
