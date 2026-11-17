import { $, type AppContext } from "../../shared/app";
import type { TransportState } from "../../shared/transport";
import { openSettings } from "../../shared/shell";
import { linkLabel } from "./link-label";
import { connectionHtml } from "./template";

/** Connect/disconnect control and the live link + key state. Mount it into the title bar. */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = document.createElement("div");
  el.className = "link";
  el.innerHTML = connectionHtml;
  root.append(el);
  const toggle = $<HTMLButtonElement>(el, "[data-toggle]");

  let link: TransportState = "disconnected";
  const render = () => {
    const { text, tone } = linkLabel(link, ctx.auth.state);
    $(el, "[data-state]").textContent = text;
    el.dataset.tone = tone;
    toggle.textContent = link === "disconnected" ? "CONNECT" : "DISCONNECT";
    toggle.disabled = link === "connecting";
  };

  $<HTMLButtonElement>(el, "[data-lock]").onclick = () => openSettings("connection");
  toggle.onclick = () => (link === "disconnected" ? ctx.transport.connect().catch((e) => ctx.log(String(e))) : ctx.transport.disconnect());
  ctx.onState((s) => {
    link = s;
    render();
  });
  ctx.auth.onChange(render);
  render();
}
