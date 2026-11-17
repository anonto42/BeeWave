import { $, download, panel, type AppContext } from "../../shared/app";
import { diffSnapshots, type SnapshotDiff } from "../../shared/monitor";
import { compareHtml, diffHtml, optionsHtml } from "./template";

/** Diff of two saved scans (new, disappeared, signal changed, channel changed), exportable as JSON. */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("COMPARE SCANS", compareHtml);
  root.append(el);
  const a = $<HTMLSelectElement>(el, "[data-a]");
  const b = $<HTMLSelectElement>(el, "[data-b]");
  const exp = $<HTMLButtonElement>(el, "[data-export]");
  const scans = ctx.monitor.scans;
  let last: { from: number; to: number; diff: SnapshotDiff } | null = null;
  /** Until the user picks something, keep comparing the second newest scan with the newest. */
  let touched = false;

  const render = () => {
    const shots = scans.snapshots();
    const keepA = touched ? a.value : "";
    const keepB = touched ? b.value : "";
    a.innerHTML = b.innerHTML = optionsHtml(shots);
    a.value = keepA || String(shots[shots.length - 2]?.id ?? shots[0]?.id ?? "");
    b.value = keepB || String(shots[shots.length - 1]?.id ?? "");
    const from = shots.find((s) => s.id === Number(a.value));
    const to = shots.find((s) => s.id === Number(b.value));
    if (!from || !to || shots.length < 2) {
      last = null;
      $(el, "[data-diff]").innerHTML = `<div class="pad dim">${shots.length < 2 ? "Save at least two scans to compare." : ""}</div>`;
    } else {
      const diff = diffSnapshots(from, to);
      last = { from: from.id, to: to.id, diff };
      $(el, "[data-diff]").innerHTML = diffHtml(diff);
    }
    exp.disabled = !last;
  };

  a.onchange = b.onchange = () => {
    touched = true;
    render();
  };
  $<HTMLButtonElement>(el, "[data-snap]").onclick = () => {
    scans.snapshot(`Snapshot ${new Date().toLocaleTimeString()}`);
  };
  exp.onclick = () => {
    if (!last) return;
    const shots = scans.snapshots();
    const body = { from: shots.find((s) => s.id === last!.from), to: shots.find((s) => s.id === last!.to), diff: last.diff };
    download("beewave-scan-comparison.json", "application/json", JSON.stringify(body, null, 2));
  };
  scans.onChange(render);
  render();
}
