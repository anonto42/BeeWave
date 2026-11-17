import { esc } from "../../shared/app";
import { sparklinePoints, type DeltaRow, type NetworkRecord } from "../../shared/monitor";

export type View = "all" | "delta";

export const historyHtml = `<div class="toolbar">
  <div class="seg" role="group" aria-label="View">
    <button class="seg-btn" data-view="all" aria-pressed="true">ALL</button>
    <button class="seg-btn" data-view="delta" aria-pressed="false">MOVED 6 dB+</button>
  </div>
</div>
<ul class="nets" data-list></ul>
<div class="pad dim small" data-note></div>`;

export const sparkSvg = (values: readonly number[]) =>
  `<svg class="spark" viewBox="0 0 90 24" aria-hidden="true"><polyline points="${sparklinePoints(values, 90, 24)}"/></svg>`;

export const rowHtml = (r: NetworkRecord, delta: number | null) =>
  `<li class="net static"><span class="net-main"><b>${esc(r.ssid || "(hidden)")}</b><small>${r.bssid} · CH ${r.channel}</small></span>` +
  sparkSvg(r.history.map((p) => p.rssi)) +
  `<span class="dbm">${r.rssi}</span>` +
  (delta === null ? "" : `<span class="delta ${delta >= 0 ? "up" : "down"}">${delta > 0 ? "+" : ""}${delta}</span>`) +
  `</li>`;

export const deltaRowHtml = (d: DeltaRow) => rowHtml(d.record, d.delta);
export const emptyHtml = (text: string) => `<li class="empty dim">${text}</li>`;
