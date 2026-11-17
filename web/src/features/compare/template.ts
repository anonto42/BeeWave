import { esc } from "../../shared/app";
import type { SnapshotDiff, SnapshotNetwork } from "../../shared/monitor";

export const compareHtml = `<div class="pad dim small">A scan is saved automatically when you stop scanning. Pick two to see what changed.</div>
<div class="toolbar">
  <button class="small" data-snap>SNAPSHOT NOW</button>
  <button class="small" data-export disabled>EXPORT JSON</button>
</div>
<div class="pad cmp">
  <label class="field">FROM <select data-a></select></label>
  <label class="field">TO <select data-b></select></label>
</div>
<div data-diff></div>`;

const name = (n: SnapshotNetwork) => esc(n.ssid || "(hidden)");
const row = (title: string, cls: string, items: string[]) =>
  `<div class="diff ${cls}"><h3>${title} (${items.length})</h3>${items.length ? `<ul>${items.map((i) => `<li>${i}</li>`).join("")}</ul>` : `<div class="dim small">none</div>`}</div>`;

export const diffHtml = (d: SnapshotDiff) =>
  row("New", "add", d.added.map((n) => `${name(n)} <small>${n.bssid} · CH ${n.channel} · ${n.rssi} dBm</small>`)) +
  row("Disappeared", "rem", d.removed.map((n) => `${name(n)} <small>${n.bssid} · CH ${n.channel}</small>`)) +
  row("Signal changed", "chg", d.rssiChanged.map((c) => `${name(c.after)} <small>${c.before.rssi} to ${c.after.rssi} dBm (${c.delta > 0 ? "+" : ""}${c.delta} dB)</small>`)) +
  row("Channel changed", "chg", d.channelChanged.map((c) => `${name(c.after)} <small>CH ${c.before.channel} to CH ${c.after.channel}</small>`));

export const optionsHtml = (shots: ReadonlyArray<{ id: number; label: string; networks: unknown[] }>) =>
  shots.map((s) => `<option value="${s.id}">${esc(s.label)} (${s.networks.length})</option>`).join("");
