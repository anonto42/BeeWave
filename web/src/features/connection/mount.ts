import { $, panel, type AppContext } from "../../shared/app";
import { connectionHtml } from "./template";

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("CONNECTION", connectionHtml);
  root.append(el);
  const connect = $<HTMLButtonElement>(el, "[data-connect]");
  const disconnect = $<HTMLButtonElement>(el, "[data-disconnect]");

  connect.onclick = () => ctx.transport.connect().catch((e) => ctx.log(String(e)));
  disconnect.onclick = () => ctx.transport.disconnect();
  ctx.onState((s) => {
    $(el, "[data-state]").textContent = s;
    connect.disabled = s !== "disconnected";
    disconnect.disabled = s !== "connected";
  });
}
