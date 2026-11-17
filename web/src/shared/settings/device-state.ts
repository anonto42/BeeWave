import type { RejectReasonName, Telemetry } from "../protocol";

type Of<T extends Telemetry["type"]> = Extract<Telemetry, { type: T }>;

/** Read-only facts the device reported on request: identity, storage, self-test. */
export class DeviceState {
  info: Of<"deviceInfo"> | null = null;
  storage: Of<"storageInfo"> | null = null;
  selfTest: Of<"selfTest"> | null = null;
  /** True between asking for a self-test and getting the result. */
  selfTestRunning = false;
  lastRejected: RejectReasonName | null = null;
  private subs: Array<() => void> = [];

  onChange(cb: () => void): void {
    this.subs.push(cb);
  }

  ingest(m: Telemetry): void {
    if (m.type === "deviceInfo") this.info = m;
    else if (m.type === "storageInfo") this.storage = m;
    else if (m.type === "selfTest") {
      this.selfTest = m;
      this.selfTestRunning = false;
    } else if (m.type === "rejected") {
      this.lastRejected = m.reason;
      this.selfTestRunning = false;
    } else return;
    this.subs.forEach((f) => f());
  }

  startSelfTest(): void {
    this.selfTestRunning = true;
    this.selfTest = null;
    this.subs.forEach((f) => f());
  }

  reset(): void {
    this.info = this.storage = this.selfTest = null;
    this.selfTestRunning = false;
    this.lastRejected = null;
    this.subs.forEach((f) => f());
  }
}
