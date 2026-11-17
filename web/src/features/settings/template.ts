import { esc } from "../../shared/app";
import { REASON_TEXT, type ReasonName, type SaveState, type Support } from "../../shared/settings";
import type { SectionDef } from "./sections";

export const shellHtml = (sections: readonly SectionDef[]) => `
<h2>SETTINGS</h2>
<div class="banner warn" data-banner hidden></div>
<nav class="stabs" aria-label="Settings sections">${sections.map((s) => `<button type="button" class="stab" data-tab="${s.id}">${esc(s.label)}</button>`).join("")}</nav>
<div class="sbody">${sections.map((s) => `<section data-section="${s.id}" hidden></section>`).join("")}</div>
<div class="applybar" role="region" aria-label="Apply settings to the device"></div>`;

/** Why the device sections are unavailable, or null when they are fine. */
export function bannerText(support: Support, authenticated: boolean, loaded: boolean): string | null {
  if (support === "older") return "This firmware is older than the app: device settings it does not know are greyed out. Update the firmware (Settings > OTA).";
  if (support === "newer") return "This firmware is newer than the app: reload the page to get the matching app before changing device settings.";
  if (support === "unsupported") return "This firmware does not support device settings (it never answered). Update it to use the Scan, Radio, Telemetry and Alerts sections.";
  if (!authenticated) return "Device settings load after the key handshake succeeds. Connect first.";
  if (!loaded) return "Reading the device's settings...";
  return null;
}

export interface BarState {
  changes: number;
  saving: boolean;
  ready: boolean;
  state: SaveState;
  reason: ReasonName | null;
  revision: number;
}

export const barHtml = (b: BarState) => {
  const status = b.saving
    ? `<span class="spinner" aria-hidden="true"></span> saving...`
    : b.state === "rejected" && b.reason
      ? `<span class="bad">&#9679; not saved: ${esc(REASON_TEXT[b.reason])}</span>`
      : b.changes > 0
        ? `<span class="warn-dot">&#9679;</span> ${b.changes} unsaved change${b.changes === 1 ? "" : "s"}`
        : `<span class="dim">${b.ready ? `no changes (saved ${b.revision} time${b.revision === 1 ? "" : "s"})` : "device settings unavailable"}</span>`;
  return `<div class="applystatus">${status}</div>
<div class="row"><button type="button" data-revert ${b.changes === 0 || b.saving ? "disabled" : ""}>REVERT</button><button type="button" class="primary" data-apply ${b.changes === 0 || b.saving || !b.ready ? "disabled" : ""}>APPLY TO DEVICE</button></div>`;
};
