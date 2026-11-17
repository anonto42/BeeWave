import { $, panel, type AppContext } from "../../shared/app";
import { logHtml } from "./template";

const MAX_LINES = 200;

/** A running record of the session: link, key handshake, commands sent, scan state and device events. */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("LOG", logHtml);
  root.append(el);
  const box = $(el, ".log");

  const add = (text: string) => {
    box.querySelector("[data-empty]")?.remove();
    box.prepend(Object.assign(document.createElement("div"), { textContent: `${new Date().toLocaleTimeString()} ${text}` }));
    while (box.childElementCount > MAX_LINES) box.lastChild!.remove();
  };

  ctx.onLog(add);
  ctx.onState((s) => add(`link ${s}`));
  ctx.auth.onChange((s) => add(`key: ${s}`));
  let scanning: boolean | undefined;
  ctx.onTelemetry("status", (m) => {
    if (m.scanning === scanning) return;
    scanning = m.scanning;
    add(m.scanning ? "scan started" : "scan stopped");
  });
  ctx.onTelemetry("settings", (m) => add(`settings: channel ${m.channel === 0 ? "AUTO" : m.channel}, scan at boot ${m.autoScan ? "on" : "off"}`));
  ctx.onTelemetry("event", (m) => add(`device event ${m.code}: ${m.message}`));
  ctx.onTelemetry("otaAck", (m) => add(`ota ack ${m.nextOffset}`));
}
