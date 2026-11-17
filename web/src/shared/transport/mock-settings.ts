import type { Command, DeviceSettings, Telemetry } from "../protocol";
import { SETTINGS_SCHEMA } from "../protocol";
import { DEFAULT_DEVICE_SETTINGS, FIELD_OF_REASON, cloneSettings, validateDeviceSettings } from "../settings";

/** What the simulated device reports about itself. */
const INFO = {
  version: "0.1.0-mock",
  gitSha: "mock000",
  built: "2026-11-18T00:00:00Z",
  otaKeyFp: "00112233445566778899aabbccddeeff",
};

export interface MockSettingsContext {
  scanning: boolean;
}

/**
 * The simulated device's settings store: validates with the same rules as the real core (a refusal writes
 * nothing), counts revisions, and answers the settings, storage, info and self-test commands.
 */
export class MockSettings {
  settings: DeviceSettings = cloneSettings(DEFAULT_DEVICE_SETTINGS);
  revision = 0;
  private busyOta = false;

  /** `Some(replies)` if `cmd` is one of ours. `factoryReset` also returns true in `resetRequested`. */
  handle(cmd: Command, ctx: MockSettingsContext): { replies: Telemetry[]; factoryReset: boolean } | null {
    switch (cmd.type) {
      case "getSettingsAll":
        return { replies: [this.all()], factoryReset: false };
      case "saveSettingsAll": {
        const bad = validateDeviceSettings(cmd.settings);
        const reason = bad && bad.reason !== "ssidTooLong" ? bad.reason : null;
        if (bad) {
          // the real device refuses with a reason and writes nothing; "ssidTooLong" cannot arrive over the wire
          void FIELD_OF_REASON;
          return { replies: [{ type: "settingsSaved", revision: this.revision, ok: false, reason: reason ?? "storage" }], factoryReset: false };
        }
        this.settings = cloneSettings(cmd.settings);
        this.revision++;
        return { replies: [{ type: "settingsSaved", revision: this.revision, ok: true, reason: null }], factoryReset: false };
      }
      case "factoryReset":
        if (this.busyOta) return { replies: [{ type: "rejected", reason: "busy" }], factoryReset: false };
        this.settings = cloneSettings(DEFAULT_DEVICE_SETTINGS);
        this.revision = 0;
        return { replies: [], factoryReset: true };
      case "runSelfTest":
        return {
          replies: [
            ctx.scanning
              ? { type: "selfTest", radio: false, nvs: false, ble: false, otaSlot: false, busy: true }
              : { type: "selfTest", radio: true, nvs: true, ble: true, otaSlot: true, busy: false },
          ],
          factoryReset: false,
        };
      case "getStorageInfo":
        return {
          replies: [
            {
              type: "storageInfo", nvsUsed: 3040 + this.revision * 8, nvsFree: 21_536 - this.revision * 8,
              ota0Active: true, ota1Active: false, ota0Size: 1_572_864, ota1Size: 1_572_864, settingsRevision: this.revision,
            },
          ],
          factoryReset: false,
        };
      case "getDeviceInfo":
        return { replies: [{ type: "deviceInfo", ...INFO, requestConnParams: false, coexDebug: false, bonding: false }], factoryReset: false };
      default:
        return null;
    }
  }

  private all(): Telemetry {
    return { type: "settingsAll", schema: SETTINGS_SCHEMA, revision: this.revision, settings: cloneSettings(this.settings) };
  }
}
