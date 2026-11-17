import { $, coalesce, panel, type AppContext } from "../../shared/app";
import { DEFAULT_QUERY, filterAndSort, nextQuery, type ResultQuery, type SortKey } from "./filter";
import { emptyRow, headHtml, resultsHtml, rowHtml } from "./template";

/** The expanded results table: every column sortable, filterable by SSID substring and channel. */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("RESULTS", resultsHtml);
  root.append(el);
  const head = $(el, "[data-head]");
  const body = $(el, "[data-body]");
  let q: ResultQuery = { ...DEFAULT_QUERY };

  const render = () => {
    const all = ctx.monitor.scans.networks();
    const rows = filterAndSort(all, q);
    const visible = ctx.settings.phone.get().ui.visibleColumns;
    head.innerHTML = headHtml(q, visible);
    body.innerHTML = rows.length ? rows.map((r) => rowHtml(r, visible)).join("") : emptyRow(visible.length);
    $(el, "[data-count]").textContent = `${rows.length} of ${all.length} networks`;
  };
  const later = coalesce(render);

  head.onclick = (e) => {
    const key = (e.target as HTMLElement).closest<HTMLElement>("[data-sort]")?.dataset.sort as SortKey | undefined;
    if (!key) return;
    q = nextQuery(q, key);
    render();
  };
  $<HTMLInputElement>(el, "[data-filter]").oninput = (e) => {
    q = { ...q, ssid: (e.target as HTMLInputElement).value };
    render();
  };
  $<HTMLSelectElement>(el, "[data-channel]").onchange = (e) => {
    q = { ...q, channel: Number((e.target as HTMLSelectElement).value) };
    render();
  };

  ctx.monitor.scans.onChange(later);
  ctx.settings.phone.onChange(render); // columns, density and RSSI unit apply at once
  render();
}
