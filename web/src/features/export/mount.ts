import { $, download, panel, type AppContext } from "../../shared/app";
import { sessionToCsv, sessionToJson } from "../../shared/monitor";

const html = `<div class="pad dim small">Saves everything this app has seen this session (scans, alerts, log, key and link events) as a file on your phone. It is assembled here; nothing is sent to the device or anywhere else.</div>
<div class="pad row">
  <button data-json>EXPORT JSON</button>
  <button data-csv>EXPORT CSV</button>
</div>
<div class="pad dim small" data-info></div>`;

const stamp = () => new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);

/** One button each for the JSON and CSV session files. No device commands. */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("EXPORT SESSION", html);
  root.append(el);
  const { scans, alerts, session } = ctx.monitor;
  const build = () => session.build(scans.events(), alerts.list(), scans.snapshots());
  const info = (what: string) => ($(el, "[data-info]").textContent = `${what}: ${scans.events().length} scan reports, ${alerts.list().length} alerts`);

  $<HTMLButtonElement>(el, "[data-json]").onclick = () => {
    download(`beewave-session-${stamp()}.json`, "application/json", sessionToJson(build()));
    info("saved JSON");
  };
  $<HTMLButtonElement>(el, "[data-csv]").onclick = () => {
    download(`beewave-session-${stamp()}.csv`, "text/csv;charset=utf-8", sessionToCsv(build()));
    info("saved CSV");
  };
}
