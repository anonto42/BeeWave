import { $, coalesce, panel, type AppContext } from "../../shared/app";
import { clientsHtml, clientRowHtml, emptyRow } from "./template";

/** Discovered client devices table: lists stations transmitting 802.11 probe requests. */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("CLIENT STATIONS (PROBES)", clientsHtml);
  root.append(el);
  const body = $(el, "[data-body]");
  const countEl = $(el, "[data-count]");
  const clearBtn = $(el, "[data-clear]");

  const render = () => {
    const list = ctx.monitor.probes.list();
    body.innerHTML = list.length ? list.map(clientRowHtml).join("") : emptyRow;
    countEl.textContent = `${list.length} client station${list.length === 1 ? "" : "s"}`;
  };
  const later = coalesce(render);

  clearBtn.onclick = () => {
    ctx.monitor.probes.clear();
  };

  ctx.monitor.probes.onChange(later);
  render();
}
