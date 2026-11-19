import { AD_CLASSES, type AdClassName } from "../../shared/protocol";
import { sparklinePoints, type BleCounts } from "../../shared/monitor";

export const LABEL: Record<AdClassName, string> = {
  fastPair: "Fast Pair",
  ibeacon: "iBeacon",
  proximity: "Proximity pairing",
  generic: "Generic",
  findMy: "Apple Find My (AirTag)",
};

const factors = [2, 3, 5, 10, 20].map((f) => `<option value="${f}">${f}x</option>`).join("");

export const bleHtml = `<div class="pad dim small">Listens for Bluetooth advertisements (passive: it sends nothing) and counts them per class over the last minute. It shares the radio with the phone link, so it is off until you turn it on.</div>
<div class="toolbar">
  <button class="small" data-toggle disabled>TURN ON</button>
  <label class="field">SPIKE AT <select data-factor>${factors}</select></label>
  <span class="dim grow small" data-state></span>
</div>
<ul class="nets" data-list></ul>`;

const spark = (values: readonly number[]) =>
  `<svg class="spark" viewBox="0 0 90 24" aria-hidden="true"><polyline points="${sparklinePoints(values, 90, 24)}"/></svg>`;

export const rowsHtml = (latest: BleCounts | null, series: Record<AdClassName, number[]>) =>
  AD_CLASSES.map(
    (c) =>
      `<li class="net static"><span class="net-main"><b>${LABEL[c]}</b><small>per minute</small></span>${spark(series[c])}<span class="dbm">${latest ? latest[c] : "-"}</span></li>`,
  ).join("");
