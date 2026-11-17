import { esc } from "../../../shared/app";
import type { WatchEntry } from "../../../shared/monitor";

const chk = (f: string, on: boolean) => `<input type="checkbox" data-f="${f}" ${on ? "checked" : ""}>`;

export const watchRow = (e: Required<WatchEntry>) =>
  `<tr data-bssid="${e.bssid}"><td>${e.bssid}<br><small class="dim">${esc(e.ssid || "(hidden)")}</small></td>` +
  `<td><input type="text" data-f="label" value="${esc(e.label)}" maxlength="32"></td><td>${chk("appear", e.appear)}</td><td>${chk("disappear", e.disappear)}</td>` +
  `<td><input type="number" data-f="deltaDb" value="${e.deltaDb}" min="0" max="60" step="1"></td><td>${chk("enabled", e.enabled)}</td>` +
  `<td><button type="button" class="small" data-remove>DEL</button></td></tr>`;
