import { $, download, panel, type AppContext } from "../../shared/app";
import { generateWigleCsv } from "../../shared/monitor";
import { filterNetworks, type FilterKey } from "./filter";
import { sortNetworks, type SortKey } from "./sort";
import { emptyHtml, rowHtml, scannerHtml } from "./template";
import type { ScanResult } from "./types";

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("SCANNER", scannerHtml);
  root.append(el);
  const body = $(el, "[data-list]");
  const networks = new Map<string, ScanResult>();

  const sortSel = $<HTMLSelectElement>(el, "[data-sort]");
  const filterSel = $<HTMLSelectElement>(el, "[data-filter]");
  const gpsBtn = $<HTMLButtonElement>(el, "[data-gps]");

  const updateGps = () => {
    if (!ctx.gps.isSupported) {
      gpsBtn.disabled = true;
      gpsBtn.textContent = "NO GPS";
      gpsBtn.title = "Geolocation not supported by this browser";
      return;
    }
    if (ctx.gps.isActive) {
      gpsBtn.classList.add("on");
      const c = ctx.gps.coords;
      gpsBtn.textContent = c ? `GPS ±${Math.round(c.accuracy)}m` : "GPS FIX…";
      gpsBtn.title = c ? `Lat: ${c.lat.toFixed(6)}, Lon: ${c.lon.toFixed(6)}` : "Waiting for GPS fix…";
    } else {
      gpsBtn.classList.remove("on");
      gpsBtn.textContent = "GPS";
      gpsBtn.title = "Activate GPS for wardriving";
    }
  };

  gpsBtn.onclick = () => {
    if (ctx.gps.isActive) ctx.gps.stop();
    else ctx.gps.start();
    updateGps();
  };
  ctx.gps.onChange(updateGps);
  updateGps();

  $<HTMLButtonElement>(el, "[data-export-wigle]").onclick = () => {
    const list = ctx.monitor.scans.networks();
    const csv = generateWigleCsv(list, ctx.gps.coords);
    const stamp = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
    download(`wigle-beewave-${stamp}.csv`, "text/csv;charset=utf-8", csv);
  };

  const render = () => {
    const allList = [...networks.values()];
    const filtered = filterNetworks(
      allList,
      filterSel.value as FilterKey,
      (bssid) => ctx.monitor.assoc.stationsFor(bssid).length,
    );
    const sorted = sortNetworks(filtered, sortSel.value as SortKey);
    body.innerHTML = sorted.length
      ? sorted
          .map((n) => {
            const rec = ctx.monitor.scans.get(n.bssid);
            const stations = ctx.monitor.assoc.stationsFor(n.bssid);
            return rowHtml(
              n,
              ctx.targets.has(n.bssid),
              ctx.monitor.watchlist.has(n.bssid),
              rec?.wps,
              rec?.security,
              rec?.decloaked,
              rec?.phy,
              stations.length,
            );
          })
          .join("")
      : emptyHtml;

    const countText = filterSel.value === "all"
      ? `${networks.size} network${networks.size === 1 ? "" : "s"}`
      : `${filtered.length} / ${networks.size} network${networks.size === 1 ? "" : "s"}`;
    $(el, "[data-count]").textContent = countText;
  };

  sortSel.onchange = render;
  filterSel.onchange = render;
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
  ctx.monitor.assoc.onChange(render);
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
