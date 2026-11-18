import { esc } from "../../shared/app";
import { rssiHtml, type WpsRecord } from "../../shared/monitor";
import type { ScanResult } from "./types";

export const scannerHtml = `<div class="toolbar">
  <label class="field">SORT <select data-sort>
    <option value="rssi">SIGNAL</option><option value="channel">CHANNEL</option><option value="ssid">NAME</option>
  </select></label>
  <span class="dim grow" data-count>0 networks</span>
  <button class="small" data-clear>CLEAR</button>
</div>
<ul class="nets" data-list></ul>`;

export const emptyHtml = `<li class="empty dim">No networks yet. Press START SCAN.</li>`;

export const rowHtml = (n: ScanResult, isTarget: boolean, pinned: boolean, wps?: WpsRecord) => {
  const wpsBadge = wps
    ? `<span class="badge small" title="${esc(wps.manufacturer)} ${esc(wps.modelName)}">${wps.version === 0x20 ? "WPS 2.0" : "WPS"}${wps.locked ? " · LOCKED" : ""}${wps.selectedRegistrar ? " · PBC" : ""}</span>`
    : "";
  const devSubtitle = wps?.deviceName || wps?.modelName ? ` · ${esc(wps.deviceName || wps.modelName)}` : "";
  return `<li class="net${isTarget ? " target" : ""}" data-bssid="${n.bssid}" data-ssid="${esc(n.ssid)}">` +
    `<span class="mark${isTarget ? " on" : ""}" aria-hidden="true"></span>` +
    `<span class="net-main"><b>${esc(n.ssid || "(hidden)")}</b><small>${n.bssid} · CH ${n.channel}${devSubtitle}</small></span>` +
    `${wpsBadge}${rssiHtml(n.rssi)}<button type="button" class="small pin${pinned ? " on" : ""}" data-pin aria-label="Add to watchlist">${pinned ? "PINNED" : "PIN"}</button></li>`;
};
