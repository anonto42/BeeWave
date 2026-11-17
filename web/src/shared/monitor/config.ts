import type { Command } from "../protocol";
import type { MonitorConfig } from "./monitor-state";

/** The `setMonitor` command for `current` with `patch` applied (the device needs all three fields). */
export function setMonitorCommand(current: MonitorConfig, patch: Partial<MonitorConfig>): Command {
  const next = { ...current, ...patch };
  return { type: "setMonitor", deauthThreshold: next.deauthThreshold, bleEnabled: next.bleEnabled, bleSpikeFactor: next.bleSpikeFactor };
}
