import type { AppContext } from "../../shared/app";

/** What every settings section is given. */
export interface SectionEnv {
  ctx: AppContext;
  /** True when device settings can be edited: authenticated, loaded and the firmware supports them. */
  deviceReady(): boolean;
  /** Send the draft to the device (the Apply bar's job; the Storage section uses it to write defaults). */
  apply(): void;
  /** Switch to another section (the Connection section is the usual target). */
  open(section: SectionId): void;
  /** Call `cb` whenever anything a section shows may have changed (model, auth, link, device facts, phone settings). */
  onRefresh(cb: () => void): void;
}

export type SectionId = "connection" | "scan" | "radio" | "telemetry" | "alerts" | "ota" | "storage" | "system" | "ui" | "watchlist" | "transfer";

/** Each section is one feature slice: it draws into `host` and keeps itself current through `env.onRefresh`. */
export type SectionMount = (host: HTMLElement, env: SectionEnv) => void;
