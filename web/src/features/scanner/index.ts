import { $, esc, panel, type AppContext } from "../../shared/app";
import type { Telemetry } from "../../shared/protocol";

type Scan = Extract<Telemetry, { type: "scan" }>;

export function mount(root: HTMLElement, ctx: AppContext) {
  const el = panel("SCANNER", `<div class="tablewrap"><table><thead><tr><th>SSID</th><th>BSSID</th><th>CH</th><th>RSSI</th></tr></thead><tbody></tbody></table></div>`);
  root.append(el);
  const nets = new Map<string, Scan>();
  ctx.onTelemetry("scan", (m) => {
    nets.set(m.bssid, m);
    $(el, "tbody").innerHTML = [...nets.values()].sort((a, b) => b.rssi - a.rssi)
      .map((n) => `<tr><td>${esc(n.ssid || "(hidden)")}</td><td>${n.bssid}</td><td>${n.channel}</td><td>${n.rssi}</td></tr>`).join("");
  });
}
