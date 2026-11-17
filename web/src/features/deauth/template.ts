import { esc } from "../../shared/app";
import { sparklinePoints, type DeauthSource } from "../../shared/monitor";
import { vendorOf } from "../../shared/oui";

export const deauthHtml = `<div class="pad dim small">Counts deauthentication frames heard on the air, per BSSID per second. It only listens: nothing is ever transmitted.</div>
<div class="toolbar">
  <label class="field">ALERT AT <input type="number" data-threshold min="0" max="250" inputmode="numeric" /> /s</label>
  <button class="small" data-apply disabled>SET</button>
  <span class="dim grow small" data-state></span>
</div>
<ul class="nets" data-list></ul>`;

const spark = (values: readonly number[]) =>
  `<svg class="spark" viewBox="0 0 90 24" aria-hidden="true"><polyline points="${sparklinePoints(values, 90, 24)}"/></svg>`;

export const rowHtml = (s: DeauthSource) =>
  `<li class="net static"><span class="net-main"><b>${esc(s.bssid)}</b><small>${esc(vendorOf(s.bssid))} · ${s.alerts} alert${s.alerts === 1 ? "" : "s"}</small></span>` +
  `${spark(s.series)}<span class="dbm">${s.last}/s</span></li>`;

export const emptyHtml = (text: string) => `<li class="empty dim">${text}</li>`;
