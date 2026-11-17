import { notice } from "../../../shared/app";
import { RESULT_COLUMNS, type ResultColumn } from "../../../shared/settings";
import type { SectionMount } from "../env";
import { mountFields, phoneSelect, phoneSlider, phoneToggle, type Bound } from "../fields";

const COLUMN_LABEL: Record<ResultColumn, string> = { ssid: "SSID", bssid: "BSSID", vendor: "Vendor", rssi: "RSSI", channel: "Channel", firstSeen: "First seen", lastSeen: "Last seen", count: "Scans" };

/** Phone-side look and feel: saved on this phone at once, never sent to the device. */
export const mount: SectionMount = (host, env) => {
  const phone = env.ctx.settings.phone;
  const columns = document.createElement("div");
  columns.className = "frow";
  columns.innerHTML = `<div class="frow-head"><span class="frow-label">Results table columns</span></div><div class="chboxes wide">${RESULT_COLUMNS.map(
    (c) => `<label class="ch"><input type="checkbox" data-col="${c}"><span>${COLUMN_LABEL[c]}</span></label>`,
  ).join("")}</div><div class="frow-hint">At least one column stays on.</div>`;
  columns.onchange = (e) => {
    const box = e.target as HTMLInputElement;
    const col = box.dataset.col as ResultColumn;
    const cur = phone.get().ui.visibleColumns;
    const next = box.checked ? RESULT_COLUMNS.filter((c) => c === col || cur.includes(c)) : cur.filter((c) => c !== col);
    if (next.length === 0) {
      box.checked = true;
      return notice("keep at least one column");
    }
    phone.update((s) => void (s.ui.visibleColumns = next));
    notice("saved locally");
  };
  const columnField: Bound = {
    el: columns,
    sync: () => columns.querySelectorAll<HTMLInputElement>("input").forEach((i) => (i.checked = phone.get().ui.visibleColumns.includes(i.dataset.col as ResultColumn))),
  };
  mountFields(host, env, [
    phoneSelect(env, "Theme", [["system", "System"], ["light", "Light"], ["dark", "Dark"]], () => phone.get().ui.theme, (v) => phone.update((s) => void (s.ui.theme = v as never))),
    phoneSelect(env, "Signal unit", [["dbm", "dBm"], ["bars", "Bars"]], () => phone.get().ui.rssiUnit, (v) => phone.update((s) => void (s.ui.rssiUnit = v as never)),
      "Bars use the usual bands: -30 excellent, -67 very good, -70 good, -80 fair, -90 poor."),
    phoneSelect(env, "Table density", [["comfortable", "Comfortable"], ["compact", "Compact"]], () => phone.get().ui.tableDensity, (v) => phone.update((s) => void (s.ui.tableDensity = v as never))),
    columnField,
    phoneSlider(env, "Refresh rate", { min: 1, max: 10, step: 1, show: (v) => `${v} / second` }, () => phone.get().ui.refreshHz, (v) => phone.update((s) => void (s.ui.refreshHz = v)),
      "Caps how often panels repaint, whatever the device's status period is."),
    phoneToggle(env, "Auto-scroll the log", () => phone.get().ui.autoScrollLog, (v) => phone.update((s) => void (s.ui.autoScrollLog = v)), "Keep the newest line in view."),
  ]);
  phone.onChange(() => columnField.sync());
};
