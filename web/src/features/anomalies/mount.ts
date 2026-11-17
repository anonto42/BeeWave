import { $, coalesce, panel, type AppContext } from "../../shared/app";
import { visible, type FeedFilter } from "./filter";
import { emptyHtml, feedHtml, itemHtml } from "./template";

const SHOWN = 100;

/** Chronological alert feed with severity colours (yellow = warn, red = alert). */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("ALERTS", feedHtml);
  root.append(el);
  const list = $(el, "[data-list]");
  const buttons = [...el.querySelectorAll<HTMLButtonElement>("[data-filter]")];
  let filter: FeedFilter = "all";

  const render = () => {
    const items = visible(ctx.monitor.alerts.list(), filter).slice(0, SHOWN);
    list.innerHTML = items.length ? items.map(itemHtml).join("") : emptyHtml;
    for (const b of buttons) b.setAttribute("aria-pressed", String(b.dataset.filter === filter));
  };
  const later = coalesce(render, 100);

  el.querySelector(".seg")!.addEventListener("click", (e) => {
    const f = (e.target as HTMLElement).closest<HTMLElement>("[data-filter]")?.dataset.filter as FeedFilter | undefined;
    if (!f) return;
    filter = f;
    render();
  });
  $<HTMLButtonElement>(el, "[data-clear]").onclick = () => ctx.monitor.alerts.clear();
  ctx.monitor.alerts.onChange(later);
  render();
}
