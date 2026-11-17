import { esc, clock } from "../../shared/app";
import type { ResultQuery, ResultRow, SortKey } from "./filter";

const COLUMNS: Array<[SortKey, string]> = [
  ["ssid", "SSID"],
  ["bssid", "BSSID"],
  ["vendor", "VENDOR"],
  ["rssi", "RSSI"],
  ["channel", "CH"],
  ["firstSeen", "FIRST"],
  ["lastSeen", "LAST"],
  ["count", "SCANS"],
];

const channelOptions = `<option value="0">All channels</option>` + Array.from({ length: 13 }, (_, i) => `<option value="${i + 1}">Channel ${i + 1}</option>`).join("");

export const resultsHtml = `<div class="toolbar">
  <input type="text" data-filter placeholder="filter by SSID" autocomplete="off" spellcheck="false" />
  <select data-channel aria-label="Channel filter">${channelOptions}</select>
</div>
<div class="tablewrap wide"><table class="grid">
  <thead><tr data-head></tr></thead>
  <tbody data-body></tbody>
</table></div>
<div class="pad dim" data-count></div>`;

export const headHtml = (q: ResultQuery) =>
  COLUMNS.map(([key, label]) => {
    const on = q.sort === key;
    return `<th aria-sort="${on ? (q.dir === "asc" ? "ascending" : "descending") : "none"}"><button class="th" data-sort="${key}">${label}${on ? (q.dir === "asc" ? " ▲" : " ▼") : ""}</button></th>`;
  }).join("");

export const rowHtml = ({ record: r, vendor }: ResultRow) =>
  `<tr><td>${esc(r.ssid || "(hidden)")}</td><td>${r.bssid}</td><td>${esc(vendor)}</td><td>${r.rssi}</td><td>${r.channel}</td>` +
  `<td>${clock(r.firstSeen)}</td><td>${clock(r.lastSeen)}</td><td>${r.count}</td></tr>`;

export const emptyRow = `<tr><td colspan="8" class="dim">No networks match.</td></tr>`;
