import { $, panel, type AppContext } from "../../shared/app";
import { sortNetworks, type SortKey } from "./sort";
import { emptyHtml, rowHtml, scannerHtml } from "./template";
import type { ScanResult } from "./types";

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("SCANNER", scannerHtml);
  root.append(el);
  const body = $(el, "[data-list]");
  const networks = new Map<string, ScanResult>();

  const sortSel = $<HTMLSelectElement>(el, "[data-sort]");

  const render = () => {
    body.innerHTML = networks.size
      ? sortNetworks([...networks.values()], sortSel.value as SortKey)
          .map((n) => rowHtml(n, ctx.targets.has(n.bssid)))
          .join("")
      : emptyHtml;
    $(el, "[data-count]").textContent = `${networks.size} network${networks.size === 1 ? "" : "s"}`;
  };

  sortSel.onchange = render;
  $<HTMLButtonElement>(el, "[data-clear]").onclick = () => {
    networks.clear();
    render();
  };

  body.onclick = (e) => {
    const row = (e.target as HTMLElement).closest<HTMLElement>("li[data-bssid]");
    if (row) ctx.targets.toggle({ bssid: row.dataset.bssid!, ssid: row.dataset.ssid ?? "" });
  };

  ctx.targets.onChange(render);
  render();
  ctx.onTelemetry("scan", (m) => {
    networks.set(m.bssid, m);
    render();
  });
}
