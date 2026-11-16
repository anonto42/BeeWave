import { esc } from "../../shared/app";
import type { ScanResult } from "./types";

export const scannerHtml = `<div class="tablewrap"><table>
  <thead><tr><th></th><th>SSID</th><th>BSSID</th><th>CH</th><th>RSSI</th></tr></thead>
  <tbody></tbody>
</table></div>`;

export const rowHtml = (n: ScanResult, isTarget: boolean) =>
  `<tr data-bssid="${n.bssid}" data-ssid="${esc(n.ssid)}" class="${isTarget ? "target" : ""}">` +
  `<td>${isTarget ? "[x]" : "[ ]"}</td><td>${esc(n.ssid || "(hidden)")}</td><td>${n.bssid}</td><td>${n.channel}</td><td>${n.rssi}</td></tr>`;
