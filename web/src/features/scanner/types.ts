import type { Telemetry } from "../../shared/protocol";

export type ScanResult = Extract<Telemetry, { type: "scan" }>;
