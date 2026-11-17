import { setRssiUnit } from "../monitor";
import type { PhoneSettings } from "../settings";
import { setRefreshHz } from "./coalesce";

/** Apply the phone-side UI settings to the page (theme, density) and to the helpers the panels use. */
export function applyUiSettings(ui: PhoneSettings["ui"]): void {
  document.documentElement.dataset.theme = ui.theme;
  document.body.classList.toggle("compact", ui.tableDensity === "compact");
  setRssiUnit(ui.rssiUnit);
  setRefreshHz(ui.refreshHz);
}
