import { $, panel, type AppContext } from "../../shared/app";
import { chipHtml, emptyHtml, trayHtml } from "./template";

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("TARGETS", trayHtml);
  root.append(el);
  const chips = $(el, "[data-chips]");

  const render = () => {
    const list = ctx.targets.list();
    chips.innerHTML = list.length ? list.map(chipHtml).join("") : emptyHtml;
  };
  chips.onclick = (e) => {
    const chip = (e.target as HTMLElement).closest<HTMLElement>("[data-bssid]");
    if (chip && (e.target as HTMLElement).matches("[data-remove]")) ctx.targets.remove(chip.dataset.bssid!);
  };
  ctx.targets.onChange(render);
  render();
}
