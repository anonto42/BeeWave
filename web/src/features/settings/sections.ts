import { mount as alerts } from "./alerts";
import { mount as connection } from "./connection";
import type { SectionId, SectionMount } from "./env";
import { mount as transfer } from "./import-export";
import { mount as ota } from "./ota";
import { mount as radio } from "./radio";
import { mount as scan } from "./scan";
import { mount as storage } from "./storage";
import { mount as system } from "./system";
import { mount as telemetry } from "./telemetry";
import { mount as ui } from "./ui";
import { mount as watchlist } from "./watchlist";

export interface SectionDef {
  id: SectionId;
  label: string;
  mount: SectionMount;
  /** Lives on the device: greyed out while the device cannot take settings. */
  device: boolean;
}

export const SECTIONS: readonly SectionDef[] = [
  { id: "connection", label: "Connection", mount: connection, device: false },
  { id: "scan", label: "Scan", mount: scan, device: true },
  { id: "radio", label: "Radio", mount: radio, device: true },
  { id: "telemetry", label: "Telemetry", mount: telemetry, device: true },
  { id: "alerts", label: "Alerts", mount: alerts, device: true },
  { id: "ota", label: "OTA", mount: ota, device: false },
  { id: "storage", label: "Storage", mount: storage, device: false },
  { id: "system", label: "System", mount: system, device: false },
  { id: "ui", label: "UI", mount: ui, device: false },
  { id: "watchlist", label: "Watchlist", mount: watchlist, device: false },
  { id: "transfer", label: "Import/Export", mount: transfer, device: false },
];
