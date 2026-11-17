import type { SectionMount } from "../env";
import { channelBoxes, mountFields, slider, textField, toggle, type Bound } from "../fields";

/** Scan settings (device-side): at boot, re-report interval, dedup threshold, SSID filter, channel filter. */
export const mount: SectionMount = (host, env) => {
  const model = env.ctx.settings.device;
  const preview = document.createElement("div");
  preview.className = "frow-hint preview";
  const previewField: Bound = {
    el: preview,
    sync() {
      const db = model.draft?.scan.dedupRssiDb ?? 6;
      preview.textContent = `Networks re-report only when RSSI moves by ${db} dB or more.`;
    },
  };
  const allNone = (wrap: HTMLElement) => {
    const bar = document.createElement("div");
    bar.className = "row";
    bar.innerHTML = `<button type="button" class="small" data-all>ALL</button><button type="button" class="small" data-none>NONE</button>`;
    bar.querySelector<HTMLButtonElement>("[data-all]")!.onclick = () => model.set("scan.channelFilterMask", 0x1fff);
    bar.querySelector<HTMLButtonElement>("[data-none]")!.onclick = () => model.set("scan.channelFilterMask", 0);
    wrap.prepend(bar);
  };
  mountFields(host, env, [
    toggle(env, "scan.scanAtBoot", "Scan at boot", "Start scanning by itself when the device powers up."),
    slider(env, "scan.scanIntervalMs", "Re-report interval", { min: 100, max: 10_000, step: 100, show: (v) => `${v} ms` },
      "Least time between two reports of the same network. New networks are never held back. The deauth window must be at least twice this."),
    slider(env, "scan.dedupRssiDb", "Re-report threshold", { min: 1, max: 30, step: 1, show: (v) => `${v} dB` }),
    previewField,
    textField(env, "scan.ssidFilter", "SSID filter", 32, "Only report networks whose name contains this text (any case). Empty = no filter. At most 32 characters."),
    channelBoxes(env, "scan.channelFilterMask", "Channel filter", "mask", "Only report networks on the ticked channels. None ticked means all channels.", undefined, allNone,
      (v) => (v.length === 0 ? "All channels" : `Channels ${v.join(", ")}`)),
  ]);
};
