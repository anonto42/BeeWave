import { $, panel, type AppContext } from "../../shared/app";

export function mount(root: HTMLElement, ctx: AppContext) {
  const chans = Array.from({ length: 13 }, (_, i) => `<option>${i + 1}</option>`).join("");
  const el = panel("CONTROL", `<div class="pad row">
    <button data-scan disabled>START SCAN</button><button data-stop disabled>STOP</button>
    <select data-chan>${chans}</select><button data-setch disabled>SET CHANNEL</button>
    <button data-reboot disabled>REBOOT</button></div>`);
  root.append(el);
  const send = (c: Parameters<AppContext["transport"]["send"]>[0]) => ctx.transport.send(c).catch((e) => ctx.log(String(e)));
  $(el, "[data-scan]").onclick = () => send({ type: "startScan", bandMask: 1 });
  $(el, "[data-stop]").onclick = () => send({ type: "stop" });
  $(el, "[data-setch]").onclick = () => send({ type: "configureRadio", channel: Number($<HTMLSelectElement>(el, "[data-chan]").value) });
  $(el, "[data-reboot]").onclick = () => send({ type: "reboot" });
  ctx.onState((s) => el.querySelectorAll("button").forEach((b) => (b.disabled = s !== "connected")));
}
