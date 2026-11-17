import { esc } from "../../shared/app";
import type { NetworkRecord, WatchEntry } from "../../shared/monitor";

export const watchHtml = `<div class="pad dim small">Pin networks to be alerted when one appears, disappears or moves by 6 dB or more. Pins stay on this phone only; the device never sees them.</div>
<div class="toolbar">
  <select data-known aria-label="Known networks"></select>
  <button class="small" data-pin-known>PIN</button>
</div>
<div class="toolbar">
  <input type="text" data-bssid placeholder="or a BSSID: aa:bb:cc:dd:ee:ff" autocomplete="off" spellcheck="false" />
  <button class="small" data-pin-typed>PIN</button>
</div>
<ul class="nets" data-list></ul>`;

export const knownOptions = (known: readonly NetworkRecord[]) =>
  `<option value="">Pick a network heard so far</option>` +
  known.map((n) => `<option value="${n.bssid}">${esc(n.ssid || "(hidden)")} ${n.bssid}</option>`).join("");

export const rowHtml = (e: WatchEntry, now: NetworkRecord | undefined) =>
  `<li class="net static" data-bssid="${e.bssid}"><span class="net-main"><b>${esc(e.ssid || "(hidden)")}</b><small>${e.bssid}</small></span>` +
  `<span class="dbm">${now ? `${now.rssi} dBm` : "not heard"}</span><button class="small" data-remove>REMOVE</button></li>`;

export const emptyHtml = `<li class="empty dim">Nothing pinned.</li>`;
