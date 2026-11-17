import { $, coalesce, panel, type AppContext } from "../../shared/app";
import { setMonitorCommand } from "../../shared/monitor";
import { bleHtml, rowsHtml } from "./template";

/** Per-class advertisement counts per minute, with a switch and the spike sensitivity. */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("BLE MONITOR", bleHtml);
  root.append(el);
  const state = ctx.monitor.state;
  const toggle = $<HTMLButtonElement>(el, "[data-toggle]");
  const factor = $<HTMLSelectElement>(el, "[data-factor]");
  let connected = false;
  const send = (patch: Parameters<typeof setMonitorCommand>[1]) =>
    ctx.transport.send(setMonitorCommand(state.effective(), patch)).catch((e) => ctx.log(`setMonitor failed: ${e}`));

  const render = () => {
    const c = state.effective();
    const ready = connected && ctx.auth.state === "authenticated";
    toggle.disabled = !ready;
    factor.disabled = !ready;
    toggle.textContent = c.bleEnabled ? "TURN OFF" : "TURN ON";
    factor.value = String(c.bleSpikeFactor);
    $(el, "[data-state]").textContent = c.bleEnabled ? "listening" : "off";
    $(el, "[data-list]").innerHTML = rowsHtml(state.bleLatest, state.bleSeries);
  };
  const later = coalesce(render);

  toggle.onclick = () => void send({ bleEnabled: !state.effective().bleEnabled });
  factor.onchange = () => void send({ bleSpikeFactor: Number(factor.value) });
  ctx.onState((s) => {
    connected = s === "connected";
    render();
  });
  ctx.auth.onChange(render);
  state.onChange(later);
  render();
}
