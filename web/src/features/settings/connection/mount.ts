import { esc, notice } from "../../../shared/app";
import { formatKey } from "../../../shared/auth";
import type { TransportState } from "../../../shared/transport";
import { confirmButton } from "../confirm";
import type { SectionMount } from "../env";
import { mountFields, phoneToggle, readOnly } from "../fields";
import { connectionHtml } from "./template";

const STATE_TEXT: Record<string, string> = {
  "no-key": "connected, no key (the device is already claimed: paste its key)",
  idle: "connected",
  authenticating: "authenticating...",
  authenticated: "authenticated",
  failed: "rejected: wrong key",
};

/** Link and key: auth state, the device key, remember-key and auto-reconnect, rotate (build-time) and forget. */
export const mount: SectionMount = (host, env) => {
  const { ctx } = env;
  const phone = ctx.settings.phone;
  host.insertAdjacentHTML("beforeend", connectionHtml);
  const q = <T extends HTMLElement>(s: string) => host.querySelector<T>(s)!;
  const state = readOnly("Link and key", "disconnected");
  const input = q<HTMLInputElement>("[data-key]");
  let link: TransportState = "disconnected";

  q("[data-state-slot]").append(state.el);
  q<HTMLButtonElement>("[data-save]").onclick = () => {
    if (!ctx.keys.set(input.value)) return ctx.log("device key must be 64 hex characters");
    input.value = "";
    if (link === "connected") void ctx.auth.start();
    else ctx.auth.reset();
    notice("key saved");
  };
  q<HTMLButtonElement>("[data-copy]").onclick = async () => {
    const key = ctx.keys.get();
    if (!key) return notice("no key to copy");
    try {
      await navigator.clipboard.writeText(formatKey(key));
      notice("key copied");
    } catch {
      notice("could not copy (clipboard blocked)");
    }
  };
  q<HTMLButtonElement>("[data-disconnect]").onclick = () => ctx.transport.disconnect();

  const forget = confirmButton({
    label: "FORGET KEY",
    confirmLabel: "FORGET AND DISCONNECT",
    warning: "Removes the saved key from this phone, drops the link and clears everything collected this session. The device keeps its key: you will need to paste it to connect again.",
    onConfirm: () => {
      ctx.transport.disconnect();
      ctx.keys.clear();
      ctx.auth.reset();
      ctx.monitor.scans.clear();
      ctx.monitor.alerts.clear();
      notice("key forgotten");
    },
  });
  q("[data-forget-slot]").append(forget.el);

  const rotate = confirmButton({
    label: "ROTATE DEVICE KEY",
    confirmLabel: "SHOW HOW",
    warning: "The key cannot be changed over Bluetooth. Rotating means rebuilding the firmware with a new key and reflashing; every other phone loses access until it is given the new key.",
    onConfirm: () => (q("[data-rotate-info]").hidden = false),
  });
  q("[data-rotate-slot]").append(rotate.el);

  mountFields(q("[data-toggles]"), env, [
    phoneToggle(env, "Remember key on this phone", () => phone.get().session.rememberKey, (v) => phone.update((s) => void (s.session.rememberKey = v)),
      "Off = the key is held in memory for this session only and is gone when the page closes."),
    phoneToggle(env, "Reconnect automatically", () => phone.get().session.autoReconnect, (v) => phone.update((s) => void (s.session.autoReconnect = v)),
      "After an unexpected drop, try to reconnect a few times. Tapping Disconnect never reconnects."),
  ]);

  const refresh = () => {
    state.set(link === "disconnected" ? `disconnected${phone.get().session.lastDeviceName ? ` (last device: ${phone.get().session.lastDeviceName})` : ""}` : link === "connecting" ? "connecting..." : STATE_TEXT[ctx.auth.state]);
    q<HTMLButtonElement>("[data-disconnect]").disabled = link === "disconnected";
    q("[data-key-state]").innerHTML = ctx.keys.get() ? "A key is saved." : `<span class="dim">No key yet: it is created automatically on the first connect.</span>`;
  };
  ctx.onState((s) => {
    link = s;
    refresh();
  });
  env.onRefresh(refresh);
  refresh();
  void esc;
};
