/** Ask the shell to show a tab: `detail` is the tab id. Lets one feature open another's screen without importing it. */
export const NAVIGATE_EVENT = "beewave:navigate";

/** Ask the Settings feature to show one of its sections: `detail` is the section id. */
export const OPEN_SETTINGS_SECTION_EVENT = "beewave:open-settings-section";

/** Open Settings > `section` (used by the lock icon in the top bar). */
export function openSettings(section: string): void {
  window.dispatchEvent(new CustomEvent(NAVIGATE_EVENT, { detail: "settings" }));
  window.dispatchEvent(new CustomEvent(OPEN_SETTINGS_SECTION_EVENT, { detail: section }));
}
