import { $, panel, type AppContext } from "../../shared/app";
import { logHtml } from "./template";

const MAX_LINES = 100;

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("LOG", logHtml);
  root.append(el);
  const box = $(el, ".log");

  const add = (text: string) => {
    box.prepend(Object.assign(document.createElement("div"), { textContent: `${new Date().toLocaleTimeString()} ${text}` }));
    while (box.childElementCount > MAX_LINES) box.lastChild!.remove();
  };

  ctx.onLog(add);
  ctx.onTelemetry("event", (m) => add(`event ${m.code}: ${m.message}`));
  ctx.onTelemetry("otaAck", (m) => add(`ota ack ${m.nextOffset}`));
}
