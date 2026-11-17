import { formatKey, type AuthState } from "../../shared/auth";
import { $, panel, type AppContext } from "../../shared/app";
import { authHtml } from "./template";

const LABEL: Record<AuthState, string> = {
  "no-key": "no key yet: created automatically on first connect",
  idle: "key saved, not connected",
  authenticating: "authenticating...",
  authenticated: "authenticated",
  failed: "rejected: wrong key?",
};

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("DEVICE KEY", authHtml);
  root.append(el);
  const input = $<HTMLInputElement>(el, "[data-key]");
  let connected = false;
  const show = (s: AuthState) =>
    ($(el, "[data-auth-state]").textContent = s === "no-key" && connected ? "device already claimed: paste its key" : LABEL[s]);
  ctx.onState((s) => {
    connected = s === "connected";
    show(ctx.auth.state);
  });
  show(ctx.auth.state);
  ctx.auth.onChange(show);
  $<HTMLButtonElement>(el, "[data-save]").onclick = () => {
    if (!ctx.keys.set(input.value)) return ctx.log("device key must be 64 hex characters");
    input.value = "";
    if (connected) void ctx.auth.start();
    else ctx.auth.reset();
  };
  const copy = $<HTMLButtonElement>(el, "[data-copy]");
  copy.onclick = async () => {
    const key = ctx.keys.get();
    if (!key) return ctx.log("no key to copy yet");
    try {
      await navigator.clipboard.writeText(formatKey(key));
      copy.textContent = "COPIED";
      setTimeout(() => (copy.textContent = "COPY KEY"), 1500);
    } catch {
      ctx.log("could not copy the key (clipboard blocked)");
    }
  };
  $<HTMLButtonElement>(el, "[data-forget]").onclick = () => {
    ctx.keys.clear();
    ctx.auth.reset();
  };
}
