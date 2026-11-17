import { clock, esc } from "../../shared/app";
import type { AlertRecord } from "../../shared/monitor";
import { FILTERS } from "./filter";

export const feedHtml = `<div class="toolbar">
  <div class="seg" role="group" aria-label="Filter">${FILTERS.map(([k, l], i) => `<button class="seg-btn" data-filter="${k}" aria-pressed="${i === 0}">${l}</button>`).join("")}</div>
  <button class="small" data-clear>CLEAR</button>
</div>
<ul class="feed" data-list></ul>`;

export const itemHtml = (a: AlertRecord) =>
  `<li class="alert sev-${a.severity}"><div class="alert-top"><b>${esc(a.title)}</b><small>${clock(a.at)} · ${a.source}</small></div><div class="alert-detail">${esc(a.detail)}</div></li>`;

export const emptyHtml = `<li class="empty dim">No alerts yet. Anomalies, deauth floods, BLE spikes and watchlist hits appear here.</li>`;
