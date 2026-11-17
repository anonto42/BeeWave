import { $, coalesce, panel, type AppContext } from "../../shared/app";
import { deltaView, lastDelta } from "../../shared/monitor";
import { deltaRowHtml, emptyHtml, historyHtml, rowHtml, type View } from "./template";

const MAX_ROWS = 25;

/**
 * Per-network RSSI sparklines over the scan cycles, plus a view of the networks that moved 6 dB or more.
 * The device only re-reports a network when it moved that much, so every extra point is such a move.
 */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("RSSI HISTORY", historyHtml);
  root.append(el);
  const list = $(el, "[data-list]");
  const buttons = [...el.querySelectorAll<HTMLButtonElement>("[data-view]")];
  let view: View = "all";

  const render = () => {
    const scans = ctx.monitor.scans;
    if (view === "delta") {
      const rows = deltaView(scans.networks(), scans.cycle);
      list.innerHTML = rows.length ? rows.slice(0, MAX_ROWS).map(deltaRowHtml).join("") : emptyHtml("Nothing moved 6 dB or more in the latest cycle.");
    } else {
      const recent = scans.networks().sort((a, b) => b.lastSeen - a.lastSeen).slice(0, MAX_ROWS);
      list.innerHTML = recent.length ? recent.map((r) => rowHtml(r, lastDelta(r))).join("") : emptyHtml("No networks yet. Start a scan.");
    }
    $(el, "[data-note]").textContent = `cycle ${scans.cycle}`;
    for (const b of buttons) b.setAttribute("aria-pressed", String(b.dataset.view === view));
  };
  const later = coalesce(render, 250);

  el.querySelector(".seg")!.addEventListener("click", (e) => {
    const v = (e.target as HTMLElement).closest<HTMLElement>("[data-view]")?.dataset.view as View | undefined;
    if (!v) return;
    view = v;
    render();
  });
  ctx.monitor.scans.onChange(later);
  render();
}
