import { esc } from "../../shared/app";
import type { Target } from "../../shared/targets";

export const trayHtml = `<div class="pad chips" data-chips></div>`;

export const chipHtml = (t: Target) =>
  `<span class="chip" data-bssid="${t.bssid}">${esc(t.ssid || t.bssid)} <button data-remove title="remove">x</button></span>`;

export const emptyHtml = `<span class="dim">click a network in the scanner to mark it as a target</span>`;
