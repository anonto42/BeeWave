import { CHANNELS } from "../../shared/monitor";

const W = 280;
const H = 130;
const PAD = { l: 6, r: 6, t: 8, b: 22 };

/**
 * Two series per channel as inline SVG: access points (white bar) and reports in the last minute (green bar).
 * Each series is scaled to its own maximum so a few busy channels do not flatten the other series.
 */
export function chartSvg(occupancy: readonly number[], activity: readonly number[]): string {
  const innerW = W - PAD.l - PAD.r;
  const innerH = H - PAD.t - PAD.b;
  const slot = innerW / CHANNELS.length;
  const maxOcc = Math.max(1, ...occupancy);
  const maxAct = Math.max(1, ...activity);
  const bar = (x: number, v: number, max: number, cls: string, w: number) => {
    const h = (v / max) * innerH;
    return `<rect class="${cls}" x="${x.toFixed(1)}" y="${(PAD.t + innerH - h).toFixed(1)}" width="${w.toFixed(1)}" height="${h.toFixed(1)}"/>`;
  };
  const bars = CHANNELS.map((ch, i) => {
    const x = PAD.l + i * slot;
    const w = (slot - 3) / 2;
    return (
      bar(x + 1, occupancy[i] ?? 0, maxOcc, "bar-ap", w) +
      bar(x + 1 + w + 1, activity[i] ?? 0, maxAct, "bar-act", w) +
      `<text class="axis" x="${(x + slot / 2).toFixed(1)}" y="${H - 8}" text-anchor="middle">${ch}</text>`
    );
  }).join("");
  const base = `<line class="axis-line" x1="${PAD.l}" x2="${W - PAD.r}" y1="${PAD.t + innerH}" y2="${PAD.t + innerH}"/>`;
  return `<svg viewBox="0 0 ${W} ${H}" role="img" aria-label="Access points and activity per channel">${base}${bars}</svg>`;
}
