import { $, panel, type AppContext } from "../../shared/app";
import { otaHtml } from "./template";
import { OtaUploader } from "./uploader";

/** `Event` code the device uses for OTA failures (EVENT_OTA_ERROR in crates/core). */
const OTA_ERROR_EVENT = 2;

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("FIRMWARE UPDATE", otaHtml);
  root.append(el);
  const file = $<HTMLInputElement>(el, "[data-file]");
  const sig = $<HTMLInputElement>(el, "[data-sig]");
  const start = $<HTMLButtonElement>(el, "[data-start]");
  const bar = $<HTMLProgressElement>(el, "[data-bar]");
  const status = $(el, "[data-status]");

  const uploader = new OtaUploader((cmd) => ctx.transport.send(cmd));
  ctx.onTelemetry("otaAck", (m) => uploader.handleAck(m.nextOffset));
  ctx.onTelemetry("event", (m) => m.code === OTA_ERROR_EVENT && uploader.handleError(m.message));

  let connected = false;
  let busy = false;
  const refresh = () => (start.disabled = !connected || busy || !file.files?.length || !sig.files?.length);
  file.onchange = refresh;
  sig.onchange = refresh;
  ctx.onState((s) => {
    connected = s === "connected";
    refresh();
  });

  start.onclick = async () => {
    const f = file.files?.[0];
    const s = sig.files?.[0];
    if (!f || !s) return;
    busy = true;
    refresh();
    try {
      const image = new Uint8Array(await f.arrayBuffer());
      const signature = new Uint8Array(await s.arrayBuffer());
      status.textContent = "uploading...";
      await uploader.upload(image, signature, (sent, total) => {
        bar.value = (sent / total) * 100;
        status.textContent = `${Math.round((sent / total) * 100)}%  (${(sent / 1024).toFixed(0)} / ${(total / 1024).toFixed(0)} KB)`;
      });
      status.textContent = "update written and verified, rebooting into the new firmware...";
      ctx.log("OTA complete, rebooting device");
      await ctx.transport.send({ type: "reboot" });
    } catch (e) {
      status.textContent = `failed: ${String(e)}`;
      ctx.log(`OTA failed: ${String(e)}`);
    } finally {
      busy = false;
      refresh();
    }
  };
}
