import { esc } from "../../shared/app";
import type { ScanResult } from "./types";

export const scannerHtml = `<div class="tablewrap"><table>
  <thead><tr><th>SSID</th><th>BSSID</th><th>CH</th><th>RSSI</th></tr></thead>
  <tbody></tbody>
</table></div>`;

export const rowHtml = (n: ScanResult) =>
  `<tr><td>${esc(n.ssid || "(hidden)")}</td><td>${n.bssid}</td><td>${n.channel}</td><td>${n.rssi}</td></tr>`;
