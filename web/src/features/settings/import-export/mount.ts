import { download, esc, notice } from "../../../shared/app";
import { fullEntry, generateWigleCsv, sessionToCsv, sessionToJson } from "../../../shared/monitor";
import {
  buildDeviceExport, buildPhoneExport, parseDeviceImport, parsePhoneImport, type WatchlistEntryExport,
} from "../../../shared/settings";
import type { SectionMount } from "../env";
import { mountFields, phoneSelect, phoneToggle } from "../fields";

const stamp = () => new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
const readFile = (input: HTMLInputElement): Promise<string | null> => {
  const f = input.files?.[0];
  input.value = "";
  return f ? f.text() : Promise.resolve(null);
};

/** Export and import settings and the session. Imports show what will change first; nothing applies blindly. */
export const mount: SectionMount = (host, env) => {
  const { ctx } = env;
  const phone = ctx.settings.phone;
  const { watchlist, scans, alerts, session } = ctx.monitor;
  host.insertAdjacentHTML("beforeend", `
    <div data-options></div>
    <div class="frow"><div class="frow-head"><span class="frow-label">Phone settings</span></div>
      <div class="row"><button type="button" data-export-phone>EXPORT</button><label class="filebtn">IMPORT<input type="file" accept="application/json,.json" data-import-phone hidden></label></div>
      <div class="frow-hint">One JSON file: every phone-side setting and the watchlist, plus the session log and scan history if ticked above. Nothing from the device except what this app already received.</div>
      <div data-phone-preview></div></div>
    <div class="frow"><div class="frow-head"><span class="frow-label">Device settings</span></div>
      <div class="row"><button type="button" data-export-device>EXPORT DEVICE SETTINGS</button><label class="filebtn">IMPORT<input type="file" accept="application/json,.json" data-import-device hidden></label></div>
      <div class="frow-hint">A separate file with only the values stored on the device, to set up several boards from one file. Importing loads it into the editor; APPLY TO DEVICE then sends it in one save.</div>
      <div data-device-preview></div></div>
    <div class="frow"><div class="frow-head"><span class="frow-label">Session</span></div>
      <div class="row"><button type="button" data-export-session>EXPORT SESSION</button></div>
      <div class="frow-hint" data-session-info>Saves this session (scans, alerts, log, key and link events) in the format chosen above, assembled on the phone.</div></div>`);
  const q = <T extends HTMLElement>(s: string) => host.querySelector<T>(s)!;
  mountFields(q("[data-options]"), env, [
    phoneSelect(env, "Session export format", [["both", "JSON and CSV"], ["json", "JSON"], ["csv", "CSV"], ["wigle", "WiGLE CSV (Wardriving)"]], () => phone.get().export.format, (v) => phone.update((s) => void (s.export.format = v as never))),
    phoneToggle(env, "Include the log", () => phone.get().export.includeLog, (v) => phone.update((s) => void (s.export.includeLog = v))),
    phoneToggle(env, "Include scan history", () => phone.get().export.includeScanHistory, (v) => phone.update((s) => void (s.export.includeScanHistory = v))),
  ]);

  const pins = (): WatchlistEntryExport[] => watchlist.list().map(fullEntry);

  // ---- phone settings
  q<HTMLButtonElement>("[data-export-phone]").onclick = () => {
    const x = phone.get().export;
    const data = session.build(x.includeScanHistory ? scans.events() : [], alerts.list(), scans.snapshots());
    const file = buildPhoneExport(phone.get(), pins(), {
      sessionLog: x.includeLog ? data.log : undefined,
      scanHistory: x.includeScanHistory ? [...scans.events()] : undefined,
    });
    download(`beewave-phone-settings-${stamp()}.json`, "application/json", JSON.stringify(file, null, 2));
  };
  q<HTMLInputElement>("[data-import-phone]").onchange = async (e) => {
    const text = await readFile(e.target as HTMLInputElement);
    if (text === null) return;
    const out = parsePhoneImport(text, phone.get(), pins());
    const slot = q("[data-phone-preview]");
    if (!out.ok) {
      slot.innerHTML = `<div class="banner err"><b>Not imported.</b><ul>${out.errors.map((m) => `<li>${esc(m)}</li>`).join("")}</ul></div>`;
      return;
    }
    const rows = out.changes.map((c) => `<li><code>${esc(c.path)}</code>: ${esc(JSON.stringify(c.from))} → ${esc(JSON.stringify(c.to))}</li>`);
    if (out.watchlistChanged) rows.push(`<li>watchlist: ${watchlist.list().length} → ${out.watchlist.length} entries</li>`);
    slot.innerHTML = rows.length
      ? `<div class="banner"><b>This will change:</b><ul>${rows.join("")}</ul><div class="row"><button type="button" data-apply-import>APPLY IMPORT</button><button type="button" data-cancel-import>CANCEL</button></div></div>`
      : `<div class="banner">Nothing to change: this file matches the current settings.</div>`;
    slot.querySelector<HTMLButtonElement>("[data-apply-import]")?.addEventListener("click", () => {
      phone.replace(out.settings);
      if (out.watchlistChanged) watchlist.replaceAll(out.watchlist);
      slot.innerHTML = "";
      notice("imported");
    });
    slot.querySelector<HTMLButtonElement>("[data-cancel-import]")?.addEventListener("click", () => (slot.innerHTML = ""));
  };

  // ---- device settings
  q<HTMLButtonElement>("[data-export-device]").onclick = () => {
    const s = ctx.settings.device.confirmed;
    if (!s) return notice("connect first: the device has not sent its settings");
    download(`beewave-device-settings-${stamp()}.json`, "application/json", JSON.stringify(buildDeviceExport(s), null, 2));
  };
  q<HTMLInputElement>("[data-import-device]").onchange = async (e) => {
    const text = await readFile(e.target as HTMLInputElement);
    if (text === null) return;
    const slot = q("[data-device-preview]");
    const out = parseDeviceImport(text);
    if (!out.ok) {
      slot.innerHTML = `<div class="banner err"><b>Not imported.</b><ul>${out.errors.map((m) => `<li>${esc(m)}</li>`).join("")}</ul></div>`;
      return;
    }
    if (!env.deviceReady()) {
      slot.innerHTML = `<div class="banner err">Connect and authenticate first, so the changes can be compared with the device.</div>`;
      return;
    }
    ctx.settings.device.setAll(out.settings);
    const n = ctx.settings.device.dirtyCount();
    slot.innerHTML = `<div class="banner">${n ? `${n} change${n === 1 ? "" : "s"} loaded into the editor. Review them in the sections, then APPLY TO DEVICE.` : "The file matches what the device already has."}</div>`;
  };

  // ---- session
  q<HTMLButtonElement>("[data-export-session]").onclick = () => {
    const x = phone.get().export;
    if (x.format === "wigle") {
      download(`wigle-beewave-${stamp()}.csv`, "text/csv;charset=utf-8", generateWigleCsv(scans.networks(), ctx.gps.coords));
      q("[data-session-info]").textContent = `Saved: ${scans.networks().length} networks in WiGLE CSV format.`;
      return;
    }
    const data = session.build(x.includeScanHistory ? scans.events() : [], alerts.list(), scans.snapshots());
    if (!x.includeLog) data.log = [];
    if (x.format !== "csv") download(`beewave-session-${stamp()}.json`, "application/json", sessionToJson(data));
    if (x.format !== "json") download(`beewave-session-${stamp()}.csv`, "text/csv;charset=utf-8", sessionToCsv(data));
    q("[data-session-info]").textContent = `Saved: ${data.scans.length} scan reports, ${data.alerts.length} alerts, ${data.log.length} log lines.`;
  };
};
