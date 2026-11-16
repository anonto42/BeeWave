import { $, panel, type AppContext } from "../../shared/app";
import { settingsHtml } from "./template";

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("SETTINGS (SAVED IN DEVICE FLASH)", settingsHtml);
  root.append(el);
  const channel = $<HTMLSelectElement>(el, "[data-channel]");
  const auto = $<HTMLInputElement>(el, "[data-autoscan]");
  const save = $<HTMLButtonElement>(el, "[data-save]");
  const status = $(el, "[data-status]");

  ctx.onTelemetry("settings", (m) => {
    channel.value = String(m.channel);
    auto.checked = m.autoScan;
    status.textContent = "saved on device";
  });

  save.onclick = () => {
    status.textContent = "saving...";
    ctx.transport
      .send({ type: "setSettings", channel: Number(channel.value), autoScan: auto.checked })
      .catch((e) => ctx.log(String(e)));
  };

  ctx.onState((s) => {
    save.disabled = s !== "connected";
  });
  // Ask for the saved settings only once the device has accepted our key; before that it refuses commands.
  ctx.auth.onChange((s) => {
    if (s === "authenticated") ctx.transport.send({ type: "getSettings" }).catch((e) => ctx.log(String(e)));
  });
}
