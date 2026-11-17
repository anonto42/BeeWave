import { $, panel, type AppContext } from "../../shared/app";
import { rebootHtml } from "./template";

/** How long the confirm step stays armed. */
const ARM_MS = 5000;

/** Reboot behind a two-step confirm (no browser dialogs). */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("DEVICE", rebootHtml);
  root.append(el);
  const reboot = $<HTMLButtonElement>(el, "[data-reboot]");
  const confirm = $<HTMLButtonElement>(el, "[data-confirm]");
  const cancel = $<HTMLButtonElement>(el, "[data-cancel]");
  const hint = $(el, "[data-hint]");
  let timer: ReturnType<typeof setTimeout> | undefined;

  const arm = (on: boolean) => {
    clearTimeout(timer);
    reboot.hidden = on;
    confirm.hidden = cancel.hidden = !on;
    hint.hidden = on;
    if (on) timer = setTimeout(() => arm(false), ARM_MS);
  };

  reboot.onclick = () => arm(true);
  cancel.onclick = () => arm(false);
  confirm.onclick = () => {
    arm(false);
    ctx.transport.send({ type: "reboot" }).catch((e) => ctx.log(String(e)));
  };
  ctx.onState((s) => {
    reboot.disabled = s !== "connected";
    if (s !== "connected") arm(false);
  });
}
