import { esc } from "../../shared/app";
import { signalBars } from "./signal";
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

export const rowHtml = (n: ScanResult, isTarget: boolean) =>
  `<li class="net${isTarget ? " target" : ""}" data-bssid="${n.bssid}" data-ssid="${esc(n.ssid)}">` +
  `<span class="mark${isTarget ? " on" : ""}" aria-hidden="true"></span>` +
  `<span class="net-main"><b>${esc(n.ssid || "(hidden)")}</b><small>${n.bssid} · CH ${n.channel}</small></span>` +
  `<span class="sig" data-bars="${signalBars(n.rssi)}" aria-hidden="true"><i></i><i></i><i></i><i></i></span>` +
  `<span class="dbm">${n.rssi}</span></li>`;
