import type { AuthState } from "../../shared/auth";
import { $, panel, type AppContext } from "../../shared/app";
import { authHtml } from "./template";

const LABEL: Record<AuthState, string> = {
  "no-key": "no key saved",
  idle: "key saved, not connected",
  authenticating: "authenticating...",
  authenticated: "authenticated",
  failed: "rejected: wrong key?",
};

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("DEVICE KEY", authHtml);
  root.append(el);
  const input = $<HTMLInputElement>(el, "[data-key]");
  const show = (s: AuthState) => ($(el, "[data-auth-state]").textContent = LABEL[s]);

  let connected = false;
  ctx.onState((s) => (connected = s === "connected"));
  show(ctx.auth.state);
  ctx.auth.onChange(show);
  $<HTMLButtonElement>(el, "[data-save]").onclick = () => {
    if (!ctx.keys.set(input.value)) return ctx.log("device key must be 64 hex characters");
    input.value = "";
    if (connected) void ctx.auth.start();
    else ctx.auth.reset();
  };
  $<HTMLButtonElement>(el, "[data-forget]").onclick = () => {
    ctx.keys.clear();
    ctx.auth.reset();
  };
}
