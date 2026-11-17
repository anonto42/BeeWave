import { esc, notice } from "../../../shared/app";
import { fullEntry } from "../../../shared/monitor";
import type { SectionMount } from "../env";
import { watchRow } from "./template";

const BSSID = /^[0-9a-f]{2}(:[0-9a-f]{2}){5}$/;

/** Pinned networks (phone-side): label, which events matter, on/off. Saved in this browser only. */
export const mount: SectionMount = (host, env) => {
  const { watchlist, scans } = env.ctx.monitor;
  host.innerHTML = `
    <div class="frow-hint">Pin networks to be alerted when one appears, disappears or moves. Pins stay on this phone; the device never sees them. Alerts appear in the Alerts feed and the Log.</div>
    <div class="grid-wrap"><table class="grid"><thead><tr><th>BSSID</th><th>LABEL</th><th>APPEAR</th><th>GONE</th><th>MOVE dB</th><th>ON</th><th></th></tr></thead><tbody data-rows></tbody></table></div>
    <div class="toolbar"><select data-known aria-label="Known networks"></select><button type="button" class="small" data-add-known>PIN</button></div>
    <div class="toolbar"><input type="text" data-bssid placeholder="BSSID aa:bb:cc:dd:ee:ff" autocomplete="off" spellcheck="false" /><input type="text" data-label placeholder="label" maxlength="32" /><button type="button" class="small" data-add-typed>ADD</button></div>`;
  const rows = host.querySelector<HTMLElement>("[data-rows]")!;
  const known = host.querySelector<HTMLSelectElement>("[data-known]")!;

  const render = () => {
    const pins = watchlist.list().map(fullEntry);
    rows.innerHTML = pins.length ? pins.map(watchRow).join("") : `<tr><td colspan="7" class="dim">Nothing pinned.</td></tr>`;
    const keep = known.value;
    known.innerHTML = `<option value="">Pin a network heard so far</option>` + scans.networks().sort((a, b) => a.ssid.localeCompare(b.ssid)).map((n) => `<option value="${n.bssid}">${esc(n.ssid || "(hidden)")} ${n.bssid}</option>`).join("");
    known.value = keep;
  };

  rows.onchange = (e) => {
    const t = e.target as HTMLInputElement;
    const bssid = t.closest<HTMLElement>("tr")?.dataset.bssid;
    if (!bssid) return;
    const field = t.dataset.f;
    if (field === "label") watchlist.update(bssid, { label: t.value.trim() });
    else if (field === "deltaDb") watchlist.update(bssid, { deltaDb: Math.max(0, Math.min(60, Math.round(Number(t.value) || 0))) });
    else if (field === "appear" || field === "disappear" || field === "enabled") watchlist.update(bssid, { [field]: t.checked });
    notice("saved locally");
  };
  rows.onclick = (e) => {
    const bssid = (e.target as HTMLElement).closest<HTMLElement>("tr")?.dataset.bssid;
    if (bssid && (e.target as HTMLElement).closest("[data-remove]")) watchlist.remove(bssid);
  };
  host.querySelector<HTMLButtonElement>("[data-add-known]")!.onclick = () => {
    const rec = scans.get(known.value);
    if (rec) watchlist.add({ bssid: rec.bssid, ssid: rec.ssid });
  };
  host.querySelector<HTMLButtonElement>("[data-add-typed]")!.onclick = () => {
    const input = host.querySelector<HTMLInputElement>("[data-bssid]")!;
    const bssid = input.value.trim().toLowerCase();
    if (!BSSID.test(bssid)) return notice("BSSID must look like aa:bb:cc:dd:ee:ff");
    watchlist.add({ bssid, ssid: scans.get(bssid)?.ssid ?? "", label: host.querySelector<HTMLInputElement>("[data-label]")!.value.trim() });
    input.value = "";
  };
  watchlist.onChange(render);
  scans.onChange(() => known.matches(":focus") || render());
  render();
  void env;
};
