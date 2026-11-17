import { esc } from "../../shared/app";
import type { Target } from "../../shared/targets";

export const trayHtml = `<ul class="nets" data-list></ul>`;

export const chipHtml = (t: Target) =>
  `<li class="net target" data-bssid="${t.bssid}"><span class="mark on" aria-hidden="true"></span>` +
  `<span class="net-main"><b>${esc(t.ssid || "(hidden)")}</b><small>${t.bssid}</small></span>` +
  `<button class="small" data-remove aria-label="Remove target">REMOVE</button></li>`;

export const emptyHtml = `<li class="empty dim">No targets. Tap a network in the scanner to add it.</li>`;
