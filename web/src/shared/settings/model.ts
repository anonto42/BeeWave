import type { DeviceSettings, Telemetry } from "../protocol";
import { SETTINGS_SCHEMA } from "../protocol";
import {
  FIELD_OF_REASON, FIELD_PATHS, cloneSettings, getField, sectionOf, validateDeviceSettings, withField,
  type FieldPath, type ReasonName, type SectionId, type Violation,
} from "./device-settings";

export type SaveState = "idle" | "saving" | "rejected";

/** Whether the connected firmware speaks this client's settings layout. */
export type Support = "unknown" | "ok" | "older" | "newer" | "unsupported";

const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);

/**
 * The device-side settings as the user edits them: the values the device last confirmed, the draft being edited,
 * and which fields differ. Nothing is sent until Apply; a rejected save changes nothing that was confirmed.
 */
export class DeviceSettingsModel {
  confirmed: DeviceSettings | null = null;
  draft: DeviceSettings | null = null;
  /** How many times the device saved its settings (proof of persistence across reboots). */
  revision = 0;
  support: Support = "unknown";
  save: { state: SaveState; reason: ReasonName | null; field: FieldPath | null } = { state: "idle", reason: null, field: null };
  private pending: DeviceSettings | null = null;
  private subs: Array<() => void> = [];

  onChange(cb: () => void): void {
    this.subs.push(cb);
  }

  get loaded(): boolean {
    return this.draft !== null;
  }

  /** Fields whose draft differs from what the device confirmed. */
  dirty(): FieldPath[] {
    const { confirmed, draft } = this;
    if (!confirmed || !draft) return [];
    return FIELD_PATHS.filter((p) => !same(getField(confirmed, p), getField(draft, p)));
  }

  dirtyCount(): number {
    return this.dirty().length;
  }

  sectionDirty(section: SectionId): number {
    return this.dirty().filter((p) => sectionOf(p) === section).length;
  }

  isDirty(p: FieldPath): boolean {
    return this.dirty().includes(p);
  }

  /** Change one field in the draft. */
  set(p: FieldPath, value: unknown): void {
    if (!this.draft) return;
    this.draft = withField(this.draft, p, value);
    if (this.save.state === "rejected" && this.save.field === p) this.save = { state: "idle", reason: null, field: null };
    this.emit();
  }

  /** Replace the whole draft (importing a device-settings file). */
  setAll(s: DeviceSettings): void {
    if (!this.draft) return;
    this.draft = cloneSettings(s);
    this.save = { state: "idle", reason: null, field: null };
    this.emit();
  }

  /** Throw away the draft and go back to what the device confirmed. */
  revert(): void {
    if (!this.confirmed) return;
    this.draft = cloneSettings(this.confirmed);
    this.save = { state: "idle", reason: null, field: null };
    this.emit();
  }

  /** The device sent its settings (the reply to `getSettingsAll`). */
  ingest(m: Telemetry): void {
    if (m.type === "settingsAll") {
      this.confirmed = cloneSettings(m.settings);
      this.draft = cloneSettings(m.settings);
      this.revision = m.revision;
      this.support = m.schema === SETTINGS_SCHEMA ? "ok" : m.schema < SETTINGS_SCHEMA ? "older" : "newer";
      this.save = { state: "idle", reason: null, field: null };
    } else if (m.type === "settingsSaved") {
      if (m.ok) {
        if (this.pending) this.confirmed = this.pending;
        this.revision = m.revision;
        this.save = { state: "idle", reason: null, field: null };
      } else {
        this.revision = m.revision;
        const reason = m.reason ?? "storage";
        this.save = { state: "rejected", reason, field: FIELD_OF_REASON[reason] };
      }
      this.pending = null;
    } else return;
    this.emit();
  }

  /** Start applying: returns the settings to send, or null if there is nothing to send. A locally detectable
   *  problem is reported without sending (the device would refuse it as well). */
  beginSave(): { send: DeviceSettings } | { violation: Violation } | null {
    if (!this.draft || this.dirtyCount() === 0 || this.save.state === "saving") return null;
    const violation = validateDeviceSettings(this.draft);
    if (violation) {
      this.save = { state: "rejected", reason: violation.reason, field: violation.field };
      this.emit();
      return { violation };
    }
    this.pending = cloneSettings(this.draft);
    this.save = { state: "saving", reason: null, field: null };
    this.emit();
    return { send: cloneSettings(this.draft) };
  }

  /** The send failed (link dropped): allow trying again. */
  saveFailed(): void {
    this.pending = null;
    this.save = { state: "idle", reason: null, field: null };
    this.emit();
  }

  markUnsupported(): void {
    if (this.support === "unknown") {
      this.support = "unsupported";
      this.emit();
    }
  }

  /** The link went down: device values are stale. Phone-side settings are not touched (they live elsewhere). */
  reset(): void {
    this.confirmed = this.draft = this.pending = null;
    this.revision = 0;
    this.support = "unknown";
    this.save = { state: "idle", reason: null, field: null };
    this.emit();
  }

  private emit(): void {
    this.subs.forEach((f) => f());
  }
}
