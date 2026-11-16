import { $, panel, type AppContext } from "../../shared/app";

export function mount(root: HTMLElement, ctx: AppContext) {
  const el = panel("LOG", `<div class="pad log"></div>`);
  root.append(el);
  const box = $(el, ".log");
  const add = (s: string) => {
    box.prepend(Object.assign(document.createElement("div"), { textContent: `${new Date().toLocaleTimeString()} ${s}` }));
    while (box.childElementCount > 100) box.lastChild!.remove();
  };
  ctx.onLog(add);
  ctx.onTelemetry("event", (m) => add(`event ${m.code}: ${m.message}`));
  ctx.onTelemetry("otaAck", (m) => add(`ota ack ${m.nextOffset}`));
}
