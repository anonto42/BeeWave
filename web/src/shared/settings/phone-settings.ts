/** Phone-side settings: kept in this browser's localStorage, never sent to the device. */
export const PHONE_SCHEMA = 1;

export const RESULT_COLUMNS = ["ssid", "bssid", "vendor", "rssi", "channel", "firstSeen", "lastSeen", "count"] as const;
export type ResultColumn = (typeof RESULT_COLUMNS)[number];

export const THEMES = ["system", "light", "dark"] as const;
export const RSSI_UNITS = ["dbm", "bars"] as const;
export const DENSITIES = ["comfortable", "compact"] as const;
export const EXPORT_FORMATS = ["json", "csv", "both", "wigle"] as const;

export interface PhoneSettings {
  ui: {
    theme: (typeof THEMES)[number];
    rssiUnit: (typeof RSSI_UNITS)[number];
    tableDensity: (typeof DENSITIES)[number];
    /** Columns shown in the Analyze results table. */
    visibleColumns: ResultColumn[];
    /** Caps how often the UI repaints, 1 to 10 times a second. */
    refreshHz: number;
    autoScrollLog: boolean;
  };
  session: {
    /** Reconnect by itself after an unexpected link drop. */
    autoReconnect: boolean;
    /** Off = the device key is held in memory for this session only. */
    rememberKey: boolean;
    lastDeviceName: string;
  };
  export: {
    format: (typeof EXPORT_FORMATS)[number];
    includeLog: boolean;
    includeScanHistory: boolean;
  };
  ota: {
    /** Firmware version the user expects; a different version on the device raises a warning. Empty = no check. */
    expectedVersion: string;
    autoCheckOnConnect: boolean;
  };
}

export const DEFAULT_PHONE_SETTINGS: PhoneSettings = {
  ui: { theme: "system", rssiUnit: "dbm", tableDensity: "comfortable", visibleColumns: [...RESULT_COLUMNS], refreshHz: 4, autoScrollLog: true },
  session: { autoReconnect: false, rememberKey: true, lastDeviceName: "" },
  export: { format: "both", includeLog: true, includeScanHistory: true },
  ota: { expectedVersion: "", autoCheckOnConnect: false },
};

export const clonePhone = (s: PhoneSettings): PhoneSettings => JSON.parse(JSON.stringify(s));

type Json = Record<string, unknown>;
const isObj = (v: unknown): v is Json => typeof v === "object" && v !== null && !Array.isArray(v);

export type Parsed = { ok: true; value: PhoneSettings } | { ok: false; errors: string[] };

/**
 * Strictly check untrusted settings (an imported file or stored text): every key must be known, every value of the
 * right type and range. Unknown keys are errors, not ignored, so a typo or a newer file never half-applies.
 * Missing keys take their defaults.
 */
export function parsePhoneSettings(input: unknown): Parsed {
  const errors: string[] = [];
  const out = clonePhone(DEFAULT_PHONE_SETTINGS);
  if (!isObj(input)) return { ok: false, errors: ["settings must be an object"] };

  const group = (name: keyof PhoneSettings, fields: Record<string, (v: unknown) => string | null>) => {
    const src = input[name];
    if (src === undefined) return;
    if (!isObj(src)) return void errors.push(`${name} must be an object`);
    for (const [k, v] of Object.entries(src)) {
      const check = fields[k];
      if (!check) errors.push(`unknown key ${name}.${k}`);
      else {
        const err = check(v);
        if (err) errors.push(`${name}.${k}: ${err}`);
        else (out[name] as unknown as Json)[k] = v;
      }
    }
  };
  const oneOf = <T extends readonly string[]>(list: T) => (v: unknown) => (typeof v === "string" && list.includes(v) ? null : `must be one of ${list.join(", ")}`);
  const bool = (v: unknown) => (typeof v === "boolean" ? null : "must be true or false");
  const text = (max: number) => (v: unknown) => (typeof v === "string" && v.length <= max ? null : `must be text of at most ${max} characters`);

  for (const k of Object.keys(input)) if (!["ui", "session", "export", "ota"].includes(k)) errors.push(`unknown key ${k}`);
  group("ui", {
    theme: oneOf(THEMES),
    rssiUnit: oneOf(RSSI_UNITS),
    tableDensity: oneOf(DENSITIES),
    visibleColumns: (v) =>
      Array.isArray(v) && v.length > 0 && v.every((c) => (RESULT_COLUMNS as readonly unknown[]).includes(c)) && new Set(v).size === v.length
        ? null
        : "must be a non-empty list of known columns without repeats",
    refreshHz: (v) => (Number.isInteger(v) && (v as number) >= 1 && (v as number) <= 10 ? null : "must be a whole number from 1 to 10"),
    autoScrollLog: bool,
  });
  group("session", { autoReconnect: bool, rememberKey: bool, lastDeviceName: text(64) });
  group("export", { format: oneOf(EXPORT_FORMATS), includeLog: bool, includeScanHistory: bool });
  group("ota", { expectedVersion: text(16), autoCheckOnConnect: bool });
  return errors.length ? { ok: false, errors } : { ok: true, value: out };
}

export interface SettingChange {
  path: string;
  from: unknown;
  to: unknown;
}

/** The settings that differ between `a` and `b`, as `group.key` paths. */
export function diffPhoneSettings(a: PhoneSettings, b: PhoneSettings): SettingChange[] {
  const out: SettingChange[] = [];
  for (const g of Object.keys(a) as Array<keyof PhoneSettings>) {
    const ga = a[g] as unknown as Json;
    const gb = b[g] as unknown as Json;
    for (const k of Object.keys(ga)) {
      if (JSON.stringify(ga[k]) !== JSON.stringify(gb[k])) out.push({ path: `${g}.${k}`, from: ga[k], to: gb[k] });
    }
  }
  return out;
}

const STORAGE_KEY = "beewave.phone-settings";

/** The settings, saved to localStorage on every change (and working in memory if storage is blocked). */
export class PhoneSettingsStore {
  private value: PhoneSettings;
  private subs: Array<(s: PhoneSettings) => void> = [];

  constructor(private readonly storage: Pick<Storage, "getItem" | "setItem" | "removeItem"> | null = safeStorage()) {
    this.value = clonePhone(DEFAULT_PHONE_SETTINGS);
    try {
      const raw = storage?.getItem(STORAGE_KEY);
      if (raw) {
        const parsed = parsePhoneSettings(JSON.parse(raw));
        if (parsed.ok) this.value = parsed.value; // corrupt or foreign data falls back to the defaults
      }
    } catch {
      /* blocked storage or bad JSON: defaults */
    }
  }

  get(): PhoneSettings {
    return this.value;
  }

  onChange(cb: (s: PhoneSettings) => void): void {
    this.subs.push(cb);
  }

  /** Change settings and save at once; no Apply step. */
  update(mutate: (draft: PhoneSettings) => void): void {
    const next = clonePhone(this.value);
    mutate(next);
    this.replace(next);
  }

  replace(next: PhoneSettings): void {
    this.value = next;
    try {
      this.storage?.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch {
      /* private mode / quota: keep working in memory */
    }
    this.subs.forEach((f) => f(next));
  }
}

function safeStorage(): Storage | null {
  try {
    return typeof localStorage === "undefined" ? null : localStorage;
  } catch {
    return null;
  }
}
