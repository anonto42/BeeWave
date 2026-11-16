import { $, panel, type AppContext } from "../../shared/app";
import { rowHtml, scannerHtml } from "./template";
import type { ScanResult } from "./types";

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("SCANNER", scannerHtml);
  root.append(el);
  const networks = new Map<string, ScanResult>();

  ctx.onTelemetry("scan", (m) => {
    networks.set(m.bssid, m);
    $(el, "tbody").innerHTML = [...networks.values()].sort((a, b) => b.rssi - a.rssi).map(rowHtml).join("");
  });
}
