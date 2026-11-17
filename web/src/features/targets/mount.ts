import { $, panel, type AppContext } from "../../shared/app";
import { chipHtml, emptyHtml, trayHtml } from "./template";

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("TARGETS", trayHtml);
  root.append(el);
  const list = $(el, "[data-list]");
  const title = $(el, "h2");

  const render = () => {
    const targets = ctx.targets.list();
    title.textContent = targets.length ? `TARGETS (${targets.length})` : "TARGETS";
    list.innerHTML = targets.length ? targets.map(chipHtml).join("") : emptyHtml;
  };
  list.onclick = (e) => {
    const row = (e.target as HTMLElement).closest<HTMLElement>("li[data-bssid]");
    if (row && (e.target as HTMLElement).closest("[data-remove]")) ctx.targets.remove(row.dataset.bssid!);
  };
  ctx.targets.onChange(render);
  render();
}
