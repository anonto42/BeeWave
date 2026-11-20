import { esc } from "../../shared/app";
import { rssiHtml, securityLabel, type SecurityRecord, type WifiPhyStandard, type WpsRecord } from "../../shared/monitor";
import type { ScanResult } from "./types";

export { securityLabel };

export const phyLabel = (phy: WifiPhyStandard): string => {
  switch (phy) {
    case "ax":
      return "Wi-Fi 6";
    case "n":
      return "Wi-Fi 4";
    case "g":
      return "802.11g";
    case "b":
      return "802.11b";
    default:
      return "";
  }
};

export const scannerHtml = `<div class="toolbar">
  <label class="field">SORT <select data-sort>
    <option value="rssi">SIGNAL</option><option value="channel">CHANNEL</option><option value="ssid">NAME</option>
  </select></label>
  <label class="field">FILTER <select data-filter>
    <option value="all">ALL</option>
    <option value="-80">&gt; -80 dBm</option>
    <option value="-70">&gt; -70 dBm</option>
    <option value="-60">&gt; -60 dBm</option>
    <option value="stations">WITH CLIENTS</option>
  </select></label>
  <span class="dim grow" data-count>0 networks</span>
  <button type="button" class="small" data-gps title="Activate GPS for wardriving">GPS</button>
  <button type="button" class="small" data-export-wigle title="Export WiGLE CSV for wardriving">WIGLE CSV</button>
  <button type="button" class="small" data-clear>CLEAR</button>
</div>
<ul class="nets" data-list></ul>`;

export const emptyHtml = `<li class="empty dim">No networks yet. Press START SCAN.</li>`;

export const rowHtml = (
  n: ScanResult,
  isTarget: boolean,
  pinned: boolean,
  wps?: WpsRecord,
  security?: SecurityRecord,
  decloaked?: boolean,
  phy?: WifiPhyStandard,
  stationCount?: number,
) => {
  const stationBadge = stationCount && stationCount > 0
    ? `<span class="badge small ok" title="${stationCount} connected client station${stationCount === 1 ? "" : "s"}">${stationCount} ${stationCount === 1 ? "CLIENT" : "CLIENTS"}</span>`
    : "";
  const decloakedBadge = decloaked ? `<span class="badge small warn">DECLOAKED</span>` : "";
  const phyBadge = phy && phy !== "unknown"
    ? `<span class="badge small${phy === "b" ? " warn" : ""}">${phyLabel(phy)}</span>`
    : "";
  const secBadge = security
    ? `<span class="badge small${security.protocol === "open" ? " warn" : security.protocol === "wep" ? " alert" : ""}">${securityLabel(security)}</span>`
    : "";
  const wpsBadge = wps
    ? `<span class="badge small" title="${esc(wps.manufacturer)} ${esc(wps.modelName)}">${wps.version === 0x20 ? "WPS 2.0" : "WPS"}${wps.locked ? " · LOCKED" : ""}${wps.selectedRegistrar ? " · PBC" : ""}</span>`
    : "";
  const devSubtitle = wps?.deviceName || wps?.modelName ? ` · ${esc(wps.deviceName || wps.modelName)}` : "";
  const badges = [stationBadge, decloakedBadge, phyBadge, secBadge, wpsBadge].filter(Boolean).join("");
  const badgesHtml = badges ? `<div class="net-badges">${badges}</div>` : "";
  return `<li class="net${isTarget ? " target" : ""}" data-bssid="${n.bssid}" data-ssid="${esc(n.ssid)}">` +
    `<span class="mark${isTarget ? " on" : ""}" aria-hidden="true"></span>` +
    `<span class="net-main"><b>${esc(n.ssid || "(hidden)")}</b><small>${n.bssid} · CH ${n.channel}${devSubtitle}</small>${badgesHtml}</span>` +
    `<span class="net-side">${rssiHtml(n.rssi)}<button type="button" class="small pin${pinned ? " on" : ""}" data-pin aria-label="Add to watchlist">${pinned ? "PINNED" : "PIN"}</button></span></li>`;
};
