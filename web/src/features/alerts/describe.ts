/** Event codes the device raises for problems (mirrors crates/core/src/app/messages.rs). */
const TITLES: Readonly<Record<number, string>> = {
  1: "Radio error",
  2: "Update failed",
  3: "Settings error",
  4: "Locked",
};

/** The alert text for a device event, or null for events that are only worth a log line. */
export function describeEvent(code: number, message: string): string | null {
  const title = TITLES[code];
  return title === undefined ? null : message ? `${title}: ${message}` : title;
}
