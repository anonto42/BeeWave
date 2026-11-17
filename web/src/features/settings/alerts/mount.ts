import type { SectionMount } from "../env";
import { mountFields, readOnly, slider } from "../fields";

/** Alert settings (device-side): when a stream of deauthentication frames counts as an alert. */
export const mount: SectionMount = (host, env) =>
  mountFields(host, env, [
    slider(env, "alerts.deauthThreshold", "Deauth threshold", { min: 1, max: 100, step: 1, show: (v) => `${v} / second` }),
    slider(env, "alerts.deauthWindowMs", "Deauth window", { min: 200, max: 5000, step: 100, show: (v) => `${v} ms` },
      "Frames are counted over this window and turned into a rate. It must be at least twice the scan re-report interval."),
    readOnly("Detection", "Receive-only. BeeWave never transmits deauth frames.", "The detector only listens; no setting can make the device transmit."),
  ]);
