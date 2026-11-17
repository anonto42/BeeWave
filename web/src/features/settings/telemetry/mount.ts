import type { SectionMount } from "../env";
import { mountFields, select, slider } from "../fields";

/** Telemetry settings (device-side): how often Status is sent, and how chatty the device's own log is. */
export const mount: SectionMount = (host, env) =>
  mountFields(host, env, [
    slider(env, "telemetry.statusPeriodMs", "Status period", { min: 250, max: 5000, step: 250, show: (v) => `${v} ms (${(1000 / v).toFixed(1)} Hz)` },
      "How often the device sends Status. Longer is gentler on the Bluetooth link."),
    select(env, "telemetry.logLevel", "Device log level", [["error", "Error"], ["warn", "Warn"], ["info", "Info"], ["debug", "Debug"]],
      "How much the device writes to its own serial log. (The Log tab on this phone shows app activity and always filters locally; the device does not send log lines over Bluetooth.)"),
  ]);
