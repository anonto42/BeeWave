import type { DeviceSettings } from "../protocol";
import { cloneSettings, validateDeviceSettings } from "./device-settings";
import { diffPhoneSettings, parsePhoneSettings, type PhoneSettings, type SettingChange } from "./phone-settings";

export const EXPORT_SCHEMA = 1;
export const PHONE_KIND = "beewave-phone-settings";
export const DEVICE_KIND = "beewave-device-settings";

export interface WatchlistEntryExport {
  bssid: string;
  ssid: string;
  label: string;
  appear: boolean;
  disappear: boolean;
  deltaDb: number;
  enabled: boolean;
}

export interface PhoneExport {
  schema: number;
  kind: typeof PHONE_KIND;
  settings: PhoneSettings;
  watchlist: WatchlistEntryExport[];
  /** Only when the owner asked for it (settings > Import / Export). */
  sessionLog?: Array<{ at: number; text: string }>;
  scanHistory?: Array<{ at: number; bssid: string; ssid: string; channel: number; rssi: number }>;
}

export function buildPhoneExport(
  settings: PhoneSettings,
  watchlist: readonly WatchlistEntryExport[],
  extras: Pick<PhoneExport, "sessionLog" | "scanHistory"> = {},
): PhoneExport {
  const out: PhoneExport = { schema: EXPORT_SCHEMA, kind: PHONE_KIND, settings, watchlist: [...watchlist] };
  if (extras.sessionLog) out.sessionLog = extras.sessionLog;
  if (extras.scanHistory) out.scanHistory = extras.scanHistory;
  return out;
}

export type PhoneImport =
  | { ok: true; settings: PhoneSettings; watchlist: WatchlistEntryExport[]; changes: SettingChange[]; watchlistChanged: boolean }
  | { ok: false; errors: string[] };

const KNOWN_TOP = ["schema", "kind", "settings", "watchlist", "sessionLog", "scanHistory"];
const BSSID = /^[0-9a-f]{2}(:[0-9a-f]{2}){5}$/;

/** Check an imported phone-settings file and work out what it would change. Nothing is applied here. */
export function parsePhoneImport(text: string, current: PhoneSettings, currentWatchlist: readonly WatchlistEntryExport[]): PhoneImport {
  let raw: unknown;
  try {
    raw = JSON.parse(text);
  } catch {
    return { ok: false, errors: ["not valid JSON"] };
  }
  if (typeof raw !== "object" || raw === null || Array.isArray(raw)) return { ok: false, errors: ["the file must contain one JSON object"] };
  const file = raw as Record<string, unknown>;
  const errors: string[] = [];
  if (file.kind !== PHONE_KIND) errors.push(`this is not a ${PHONE_KIND} file`);
  if (file.schema !== EXPORT_SCHEMA) errors.push(`unsupported schema version ${String(file.schema)} (this app reads ${EXPORT_SCHEMA})`);
  for (const k of Object.keys(file)) if (!KNOWN_TOP.includes(k)) errors.push(`unknown key ${k}`);

  const parsed = parsePhoneSettings(file.settings ?? {});
  if (!parsed.ok) errors.push(...parsed.errors);

  const watchlist: WatchlistEntryExport[] = [];
  if (file.watchlist !== undefined) {
    if (!Array.isArray(file.watchlist)) errors.push("watchlist must be a list");
    else {
      file.watchlist.forEach((e: unknown, i: number) => {
        const w = e as Record<string, unknown>;
        const bad = (why: string) => errors.push(`watchlist[${i}]: ${why}`);
        if (typeof w !== "object" || w === null) return bad("must be an object");
        for (const k of Object.keys(w)) if (!["bssid", "ssid", "label", "appear", "disappear", "deltaDb", "enabled"].includes(k)) bad(`unknown key ${k}`);
        if (typeof w.bssid !== "string" || !BSSID.test(w.bssid)) return bad("bssid must look like aa:bb:cc:dd:ee:ff");
        const deltaDb = w.deltaDb ?? 6;
        if (!Number.isInteger(deltaDb) || (deltaDb as number) < 0 || (deltaDb as number) > 60) return bad("deltaDb must be 0 to 60");
        watchlist.push({
          bssid: w.bssid,
          ssid: typeof w.ssid === "string" ? w.ssid : "",
          label: typeof w.label === "string" ? w.label : "",
          appear: w.appear !== false,
          disappear: w.disappear !== false,
          deltaDb: deltaDb as number,
          enabled: w.enabled !== false,
        });
      });
    }
  }
  if (errors.length || !parsed.ok) return { ok: false, errors };
  const changes = diffPhoneSettings(current, parsed.value);
  const watchlistChanged = file.watchlist !== undefined && JSON.stringify(watchlist) !== JSON.stringify(currentWatchlist);
  return { ok: true, settings: parsed.value, watchlist, changes, watchlistChanged };
}

export interface DeviceExport {
  schema: number;
  kind: typeof DEVICE_KIND;
  device: DeviceSettings;
}

export const buildDeviceExport = (s: DeviceSettings): DeviceExport => ({ schema: EXPORT_SCHEMA, kind: DEVICE_KIND, device: cloneSettings(s) });

export type DeviceImport = { ok: true; settings: DeviceSettings } | { ok: false; errors: string[] };

const GROUP_KEYS: Record<string, string[]> = {
  scan: ["scanAtBoot", "scanIntervalMs", "dedupRssiDb", "ssidFilter", "channelFilterMask"],
  radio: ["defaultChannel", "hopDwellMs", "hopOrder", "customHopList"],
  telemetry: ["statusPeriodMs", "logLevel"],
  alerts: ["deauthThreshold", "deauthWindowMs"],
};

/** Check a device-settings file: schema, no unknown or missing keys, and the same ranges the device enforces. */
export function parseDeviceImport(text: string): DeviceImport {
  let raw: unknown;
  try {
    raw = JSON.parse(text);
  } catch {
    return { ok: false, errors: ["not valid JSON"] };
  }
  const errors: string[] = [];
  const file = raw as Record<string, unknown> | null;
  if (typeof file !== "object" || file === null || Array.isArray(file)) return { ok: false, errors: ["the file must contain one JSON object"] };
  if (file.kind !== DEVICE_KIND) errors.push(`this is not a ${DEVICE_KIND} file`);
  if (file.schema !== EXPORT_SCHEMA) errors.push(`unsupported schema version ${String(file.schema)} (this app reads ${EXPORT_SCHEMA})`);
  for (const k of Object.keys(file)) if (!["schema", "kind", "device"].includes(k)) errors.push(`unknown key ${k}`);
  const dev = file.device as Record<string, Record<string, unknown>> | undefined;
  if (typeof dev !== "object" || dev === null) errors.push("device settings are missing");
  else {
    for (const g of Object.keys(dev)) if (!(g in GROUP_KEYS)) errors.push(`unknown key device.${g}`);
    for (const [g, keys] of Object.entries(GROUP_KEYS)) {
      const group = dev[g];
      if (typeof group !== "object" || group === null) {
        errors.push(`device.${g} is missing`);
        continue;
      }
      for (const k of Object.keys(group)) if (!keys.includes(k)) errors.push(`unknown key device.${g}.${k}`);
      for (const k of keys) if (!(k in group)) errors.push(`device.${g}.${k} is missing`);
    }
  }
  if (errors.length) return { ok: false, errors };
  const settings = cloneSettings(dev as unknown as DeviceSettings);
  const bad = validateDeviceSettings(settings);
  if (bad) return { ok: false, errors: [`${bad.field}: ${bad.reason}`] };
  return { ok: true, settings };
}
