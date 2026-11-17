import { clock, esc } from "../../shared/app";
import { rssiHtml } from "../../shared/monitor";
import type { ResultColumn } from "../../shared/settings";
import type { ResultQuery, ResultRow, SortKey } from "./filter";

const COLUMNS: Array<[SortKey & ResultColumn, string]> = [
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

export const headHtml = (q: ResultQuery, visible: readonly ResultColumn[]) =>
  COLUMNS.filter(([key]) => visible.includes(key)).map(([key, label]) => {
    const on = q.sort === key;
    return `<th aria-sort="${on ? (q.dir === "asc" ? "ascending" : "descending") : "none"}"><button class="th" data-sort="${key}">${label}${on ? (q.dir === "asc" ? " ▲" : " ▼") : ""}</button></th>`;
  }).join("");

const CELLS: Record<ResultColumn, (r: ResultRow) => string> = {
  ssid: ({ record: r }) => esc(r.ssid || "(hidden)"),
  bssid: ({ record: r }) => r.bssid,
  vendor: ({ vendor }) => esc(vendor),
  rssi: ({ record: r }) => rssiHtml(r.rssi),
  channel: ({ record: r }) => String(r.channel),
  firstSeen: ({ record: r }) => clock(r.firstSeen),
  lastSeen: ({ record: r }) => clock(r.lastSeen),
  count: ({ record: r }) => String(r.count),
};

export const rowHtml = (row: ResultRow, visible: readonly ResultColumn[]) =>
  `<tr>${COLUMNS.filter(([key]) => visible.includes(key)).map(([key]) => `<td>${CELLS[key](row)}</td>`).join("")}</tr>`;

export const emptyRow = (cols: number) => `<tr><td colspan="${cols}" class="dim">No networks match.</td></tr>`;
