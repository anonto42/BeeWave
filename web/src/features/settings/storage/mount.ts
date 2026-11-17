import type { SectionMount } from "../env";
import { DEFAULT_DEVICE_SETTINGS, cloneSettings } from "../../../shared/settings";
import { confirmButton } from "../confirm";
import { mountFields, readOnly } from "../fields";

const kb = (n: number) => (n >= 1024 ? `${(n / 1024).toFixed(1)} KB` : `${n} B`);

/** Read-only flash diagnostics, filled on demand, and the erase-settings action. */
export const mount: SectionMount = (host, env) => {
  const { ctx } = env;
  const dev = ctx.settings.state;
  const nvs = readOnly("NVS used / free", "-");
  const slots = readOnly("OTA slots", "-");
  const rev = readOnly("Settings revision", "-", "Counts every save the device made. After a power cycle it must be unchanged: that is the proof the settings persisted.");
  const bar = document.createElement("div");
  bar.className = "row";
  bar.innerHTML = `<button type="button" data-refresh>REFRESH</button><span class="dim small" data-hint>Loaded on demand.</span>`;
  const refreshBtn = bar.querySelector<HTMLButtonElement>("[data-refresh]")!;
  refreshBtn.onclick = () => ctx.transport.send({ type: "getStorageInfo" }).catch((e) => ctx.log(`getStorageInfo failed: ${e}`));

  const erase = confirmButton({
    label: "ERASE NVS SETTINGS",
    confirmLabel: "WRITE FACTORY DEFAULTS",
    warning: "Writes the factory default settings to the device. It does not reboot, and it keeps the device key. (Factory reset, under System, also erases the key and reboots.)",
    onConfirm: () => {
      ctx.settings.device.setAll(cloneSettings(DEFAULT_DEVICE_SETTINGS));
      env.apply();
    },
  });
  const wrap = document.createElement("div");
  wrap.className = "frow";
  wrap.append(Object.assign(document.createElement("div"), { className: "frow-head", innerHTML: '<span class="frow-label">Erase settings</span>' }), erase.el);

  mountFields(host, env, [nvs, slots, rev]);
  host.prepend(bar);
  host.append(wrap);

  const refresh = () => {
    const s = dev.storage;
    const ready = env.deviceReady();
    refreshBtn.disabled = !ready;
    erase.setDisabled(!ready, "Available once connected and authenticated.");
    if (!s) return;
    const total = s.nvsUsed + s.nvsFree;
    nvs.set(`${kb(s.nvsUsed)} used, ${kb(s.nvsFree)} free (${total ? Math.round((s.nvsUsed / total) * 100) : 0}%)`);
    slots.set(`ota_0 ${kb(s.ota0Size)}${s.ota0Active ? " (active)" : ""}; ota_1 ${kb(s.ota1Size)}${s.ota1Active ? " (active)" : ""}`);
    rev.set(String(s.settingsRevision));
  };
  dev.onChange(refresh);
  env.onRefresh(refresh);
  refresh();
};
