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
          .map((n) => {
            const rec = ctx.monitor.scans.get(n.bssid);
            return rowHtml(
              n,
              ctx.targets.has(n.bssid),
              ctx.monitor.watchlist.has(n.bssid),
              rec?.wps,
              rec?.security,
              rec?.decloaked,
            );
          })
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
    const pin = (e.target as HTMLElement).closest<HTMLElement>("[data-pin]");
    if (pin) {
      // one-click pin from a scanner row (the row itself marks a target)
      const li = pin.closest<HTMLElement>("li[data-bssid]")!;
      const { watchlist } = ctx.monitor;
      if (watchlist.has(li.dataset.bssid!)) watchlist.remove(li.dataset.bssid!);
      else watchlist.add({ bssid: li.dataset.bssid!, ssid: li.dataset.ssid ?? "" });
      return;
    }
    const row = (e.target as HTMLElement).closest<HTMLElement>("li[data-bssid]");
    if (row) ctx.targets.toggle({ bssid: row.dataset.bssid!, ssid: row.dataset.ssid ?? "" });
  };

  ctx.targets.onChange(render);
  ctx.monitor.watchlist.onChange(render);
  ctx.monitor.scans.onChange(render);
  render();
  ctx.onTelemetry("scan", (m) => {
    networks.set(m.bssid, m);
    render();
  });
  ctx.onTelemetry("decloakedResult", (m) => {
    networks.set(m.bssid, { type: "scan", bssid: m.bssid, ssid: m.ssid, channel: m.channel, rssi: m.rssi });
    render();
  });
}
