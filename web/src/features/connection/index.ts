import { $, panel, type AppContext } from "../../shared/app";

export function mount(root: HTMLElement, ctx: AppContext) {
  const el = panel("CONNECTION", `<div class="pad row"><button data-c>CONNECT</button><button data-d disabled>DISCONNECT</button><span data-s>disconnected</span></div>`);
  root.append(el);
  const c = $<HTMLButtonElement>(el, "[data-c]"), d = $<HTMLButtonElement>(el, "[data-d]");
  c.onclick = () => ctx.transport.connect().catch((e) => ctx.log(String(e)));
  d.onclick = () => ctx.transport.disconnect();
  ctx.onState((s) => { $(el, "[data-s]").textContent = s; c.disabled = s !== "disconnected"; d.disabled = s !== "connected"; });
}
