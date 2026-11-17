import { $, coalesce, panel, type AppContext } from "../../shared/app";
import { setMonitorCommand } from "../../shared/monitor";
import { emptyHtml, deauthHtml, rowHtml } from "./template";

/** Live table of BSSIDs that crossed the deauth threshold, with a sparkline of frames per second. */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("DEAUTH DETECTOR", deauthHtml);
  root.append(el);
  const list = $(el, "[data-list]");
  const input = $<HTMLInputElement>(el, "[data-threshold]");
  const apply = $<HTMLButtonElement>(el, "[data-apply]");
  const state = ctx.monitor.state;
  let connected = false;

  const render = () => {
    const sources = [...state.deauth.values()].sort((a, b) => b.lastAt - a.lastAt);
    const t = state.effective().deauthThreshold;
    list.innerHTML = sources.length ? sources.map(rowHtml).join("") : emptyHtml(t === 0 ? "Detector is off." : "No deauth floods heard.");
    if (document.activeElement !== input) input.value = String(t);
    $(el, "[data-state]").textContent = t === 0 ? "off" : "";
    apply.disabled = !connected || ctx.auth.state !== "authenticated";
  };
  const later = coalesce(render);

  apply.onclick = () => {
    const n = Math.max(0, Math.min(250, Math.round(Number(input.value))));
    if (Number.isNaN(n)) return;
    ctx.transport.send(setMonitorCommand(state.effective(), { deauthThreshold: n })).catch((e) => ctx.log(`setMonitor failed: ${e}`));
  };
  ctx.onState((s) => {
    connected = s === "connected";
    render();
  });
  ctx.auth.onChange(render);
  state.onChange(later);
  render();
}
