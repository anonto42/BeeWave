import { $, coalesce, panel, type AppContext } from "../../shared/app";
import { emptyHtml, knownOptions, rowHtml, watchHtml } from "./template";

const BSSID = /^[0-9a-f]{2}(:[0-9a-f]{2}){5}$/;

/** Pin BSSIDs on the phone; alerts come from the shared watchlist rules (see the Alerts feed). */
export function mount(root: HTMLElement, ctx: AppContext): void {
  const el = panel("WATCHLIST", watchHtml);
  root.append(el);
  const { watchlist, scans } = ctx.monitor;
  const known = $<HTMLSelectElement>(el, "[data-known]");
  const typed = $<HTMLInputElement>(el, "[data-bssid]");

  const render = () => {
    const keep = known.value;
    known.innerHTML = knownOptions(scans.networks().sort((a, b) => a.ssid.localeCompare(b.ssid)));
    known.value = keep;
    const pins = watchlist.list();
    $(el, "[data-list]").innerHTML = pins.length ? pins.map((p) => rowHtml(p, scans.get(p.bssid))).join("") : emptyHtml;
  };
  const later = coalesce(render, 300);

  $<HTMLButtonElement>(el, "[data-pin-known]").onclick = () => {
    const rec = scans.get(known.value);
    if (rec) watchlist.add({ bssid: rec.bssid, ssid: rec.ssid });
  };
  $<HTMLButtonElement>(el, "[data-pin-typed]").onclick = () => {
    const bssid = typed.value.trim().toLowerCase();
    if (!BSSID.test(bssid)) return ctx.log("watchlist: BSSID must look like aa:bb:cc:dd:ee:ff");
    watchlist.add({ bssid, ssid: scans.get(bssid)?.ssid ?? "" });
    typed.value = "";
  };
  $(el, "[data-list]").onclick = (e) => {
    const row = (e.target as HTMLElement).closest<HTMLElement>("li[data-bssid]");
    if (row && (e.target as HTMLElement).closest("[data-remove]")) watchlist.remove(row.dataset.bssid!);
  };
  watchlist.onChange(render);
  scans.onChange(later);
  render();
}
