import type { AppContext } from "../../shared/app";
import { describeEvent } from "./describe";

const SHOW_MS = 6000;
const MAX_TOASTS = 3;

/** Device errors as tap-to-dismiss alerts above the tab bar (they are also in the Log tab). */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const box = document.createElement("div");
  box.className = "toasts";
  box.setAttribute("role", "alert");
  root.append(box);

  ctx.onTelemetry("event", (m) => {
    const text = describeEvent(m.code, m.message);
    if (!text) return;
    const toast = Object.assign(document.createElement("div"), { className: "toast", textContent: text });
    toast.onclick = () => toast.remove();
    box.append(toast);
    while (box.childElementCount > MAX_TOASTS) box.firstElementChild!.remove();
    setTimeout(() => toast.remove(), SHOW_MS);
  });
}
