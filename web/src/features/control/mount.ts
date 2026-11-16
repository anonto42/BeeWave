import { $, panel, type AppContext } from "../../shared/app";
import type { Command } from "../../shared/protocol";
import { controlHtml } from "./template";

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("CONTROL", controlHtml);
  root.append(el);
  const send = (cmd: Command) => ctx.transport.send(cmd).catch((e) => ctx.log(String(e)));

  $(el, "[data-scan]").onclick = () => send({ type: "startScan", bandMask: 1 });
  $(el, "[data-stop]").onclick = () => send({ type: "stop" });
  $(el, "[data-set-channel]").onclick = () =>
    send({ type: "configureRadio", channel: Number($<HTMLSelectElement>(el, "[data-channel]").value) });
  $(el, "[data-reboot]").onclick = () => send({ type: "reboot" });

  ctx.onState((s) => el.querySelectorAll("button").forEach((b) => (b.disabled = s !== "connected")));
}
