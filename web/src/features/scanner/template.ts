import { esc } from "../../shared/app";
import { rssiHtml, type SecurityRecord, type WpsRecord } from "../../shared/monitor";
import type { ScanResult } from "./types";

export const scannerHtml = `<div class="toolbar">
  <label class="field">SORT <select data-sort>
    <option value="rssi">SIGNAL</option><option value="channel">CHANNEL</option><option value="ssid">NAME</option>
  </select></label>
  <span class="dim grow" data-count>0 networks</span>
  <button type="button" class="small" data-gps title="Activate GPS for wardriving">GPS</button>
  <button type="button" class="small" data-export-wigle title="Export WiGLE CSV for wardriving">WIGLE CSV</button>
  <button type="button" class="small" data-clear>CLEAR</button>
</div>
<ul class="nets" data-list></ul>`;

export const emptyHtml = `<li class="empty dim">No networks yet. Press START SCAN.</li>`;

export const securityLabel = (sec: SecurityRecord): string => {
  const protoMap: Record<string, string> = {
    open: "OPEN",
    wep: "WEP",
    wpa: "WPA",
    wpa2: "WPA2",
    wpa3: "WPA3",
    wpa2Wpa3: "WPA2/WPA3",
    owe: "OWE",
    enterprise: "802.1X",
  };
  const proto = protoMap[sec.protocol] ?? sec.protocol.toUpperCase();
  const pmf = sec.pmf === "required" ? " · PMF" : sec.pmf === "capable" ? " · PMF OPT" : "";
  return `${proto}${pmf}`;
};

export const rowHtml = (
  n: ScanResult,
  isTarget: boolean,
  pinned: boolean,
  wps?: WpsRecord,
  security?: SecurityRecord,
  decloaked?: boolean,
) => {
  const decloakedBadge = decloaked ? `<span class="badge small warn">DECLOAKED</span>` : "";
  const secBadge = security
    ? `<span class="badge small${security.protocol === "open" ? " warn" : security.protocol === "wep" ? " alert" : ""}">${securityLabel(security)}</span>`
    : "";
  const wpsBadge = wps
    ? `<span class="badge small" title="${esc(wps.manufacturer)} ${esc(wps.modelName)}">${wps.version === 0x20 ? "WPS 2.0" : "WPS"}${wps.locked ? " · LOCKED" : ""}${wps.selectedRegistrar ? " · PBC" : ""}</span>`
    : "";
  const devSubtitle = wps?.deviceName || wps?.modelName ? ` · ${esc(wps.deviceName || wps.modelName)}` : "";
  return `<li class="net${isTarget ? " target" : ""}" data-bssid="${n.bssid}" data-ssid="${esc(n.ssid)}">` +
    `<span class="mark${isTarget ? " on" : ""}" aria-hidden="true"></span>` +
    `<span class="net-main"><b>${esc(n.ssid || "(hidden)")}</b><small>${n.bssid} · CH ${n.channel}${devSubtitle}</small></span>` +
    `${decloakedBadge}${secBadge}${wpsBadge}${rssiHtml(n.rssi)}<button type="button" class="small pin${pinned ? " on" : ""}" data-pin aria-label="Add to watchlist">${pinned ? "PINNED" : "PIN"}</button></li>`;
};
