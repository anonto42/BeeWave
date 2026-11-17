import { $, panel, type AppContext } from "../../shared/app";
import type { Command } from "../../shared/protocol";
import type { TransportState } from "../../shared/transport";
import { controlState } from "./enabled";
import { controlHtml } from "./template";

export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("CONTROL", controlHtml);
  root.append(el);
  const send = (cmd: Command) => ctx.transport.send(cmd).catch((e) => ctx.log(`${cmd.type} failed: ${e}`));
  const run = $<HTMLButtonElement>(el, "[data-run]");
  const mode = $(el, "[data-mode]");
  const hint = $(el, "[data-hint]");
  const chips = [...el.querySelectorAll<HTMLButtonElement>("[data-ch]")];

  let link: TransportState = "disconnected";
  let scanning = false;
  let pinned = 0; // the channel the user picked (0 = AUTO)
  const render = () => {
    const s = controlState(link, ctx.auth.state, scanning);
    run.disabled = !s.run.enabled;
    run.textContent = s.run.label;
    run.classList.toggle("stop", scanning);
    mode.textContent = s.mode;
    mode.dataset.mode = s.mode.toLowerCase();
    for (const c of chips) {
      c.disabled = !s.channelEnabled;
      c.setAttribute("aria-pressed", String(Number(c.dataset.ch) === pinned));
    }
  };

  run.onclick = () => {
    scanning = !scanning; // optimistic: the next status (within a second) confirms it
    render();
    void send(scanning ? { type: "startScan", bandMask: 1 } : { type: "stop" });
  };
  $(el, "[data-channels]").onclick = (e) => {
    const chip = (e.target as HTMLElement).closest<HTMLButtonElement>("[data-ch]");
    if (!chip || chip.disabled) return;
    pinned = Number(chip.dataset.ch);
    render();
    void send({ type: "configureRadio", channel: pinned });
  };

  ctx.onState((s) => {
    link = s;
    if (s !== "connected") {
      scanning = false;
      hint.textContent = "";
    }
    render();
  });
  ctx.auth.onChange(render);
  ctx.onTelemetry("status", (m) => {
    scanning = m.scanning;
    hint.textContent = `on channel ${m.activeChannel}`;
    render();
  });
  render();
}
