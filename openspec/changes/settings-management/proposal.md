## Why

Only the channel and scan-at-boot are persisted today; the hop dwell and order, the dedup threshold, the status
period and the alert tuning are compile-time constants. The owner asked for a complete Settings surface with an explicit
device-side / phone-side split, atomic Apply and Revert, diagnostics and import/export.

## What Changes

- **Device-side settings** (validated in `crates/core/src/features/settings`, persisted in NVS, revision counter):
  scan (at boot, minimum re-report interval, dedup dB, SSID filter, channel mask), radio (default channel, hop dwell,
  hop order, custom hop list), telemetry (status period, log level), alerts (deauth threshold and window).
- **Wire protocol (append-only, version stays 1):** commands `GetSettingsAll`, `SaveSettingsAll`, `FactoryReset`,
  `RunSelfTest`, `GetStorageInfo`, `GetDeviceInfo`; telemetry `SettingsAll` (with a schema number), `SettingsSaved`,
  `StorageInfo`, `SelfTestResult`, `DeviceInfo`, `Rejected`; enums `HopOrder`, `LogLevel`, `RejectReason`. The old
  `GetSettings` / `SetSettings` stay and now map onto the new model.
- **Ports:** `SettingsStore` gains `save_revision` and `erase_all`; `System` gains NVS usage, partition info, build facts
  and self-test helpers; `RadioDriver` gains `configure_hop` (applied when a scan starts, never mid-hop).
- **Web:** the Settings tab becomes eleven sections with one Apply / Revert bar; phone-side settings live in `localStorage`
  and save immediately; import / export with a schema version.

## Decisions on points the request left open

- `scan_interval_ms` = minimum time between re-reports of the same network (new networks are never held back).
- `deauth_window_ms` is the period the sniffer counters are drained over; the threshold is a rate per second.
- `log_level` sets the device's own log verbosity (serial); the device sends no log events over the wire.
- `RejectReason` names the offending field so the client can highlight it; whole-request refusals use `Rejected`.
- Wire structs are plain serde types. `#[repr(C, packed)]` is not used: postcard serialises field by field (so it adds
  nothing for byte stability) and taking references to packed fields is undefined behaviour.
- `FactoryReset` also erases the claimed key; the device boots unclaimed.

## Non-goals

No new radio-TX path, no BLE advertisement sending, no bonding change, `REQUEST_CONN_PARAMS` stays false. No key rotation
or editing over the wire: the device key and the OTA signing key are build-time or claim-time only.

## Impact

`crates/protocol`, `crates/core` (settings slice, scan table, deauth window, `app/settings_commands.rs`), `firmware`
(NVS, hopper, esp_system, app task), `web` (settings feature slices, stores, import/export), docs.
