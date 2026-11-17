# Settings

The Settings tab is one panel with eleven sections. Everything is either **device-side** (stored in the board's NVS flash, validated by
the core, changed with one **Apply** and undone with **Revert**), **phone-side** (stored in this browser's `localStorage`, saved the moment
it changes, never sent to the device) or **read only** (fixed when the firmware was built). Device settings load only after the key handshake
succeeds (`AuthResult ok`); before that the sections are greyed out and nothing is requested.

## Where each setting lives

| Group | Setting | Side | Range / values | Default |
|---|---|---|---|---|
| Scan | `scan_at_boot` | device | on / off | off |
| Scan | `scan_interval_ms` | device | 100-10000 | 250 |
| Scan | `dedup_rssi_db` | device | 1-30 | 6 |
| Scan | `ssid_filter` | device | text, at most 32 bytes | empty |
| Scan | `channel_filter_mask` | device | bits 0-12 = channels 1-13; 0 = all | 0 |
| Radio | `default_channel` | device | 0 = AUTO, 1-13 | 0 |
| Radio | `hop_dwell_ms` | device | 100-2000 | 250 |
| Radio | `hop_order` | device | Ascending, 1-6-11 first, Custom | Ascending |
| Radio | `custom_hop_list` | device | 13 slots: channels 1-13 once each, 0 = unused | empty |
| Telemetry | `status_period_ms` | device | 250-5000 | 1000 |
| Telemetry | `log_level` | device | Error, Warn, Info, Debug | Info |
| Alerts | `deauth_threshold` | device | 1-100 per second | 10 |
| Alerts | `deauth_window_ms` | device | 200-5000 | 1000 |
| UI | `theme` | phone | System, Light, Dark | System |
| UI | `rssi_unit` | phone | dBm, Bars | dBm |
| UI | `table_density` | phone | Comfortable, Compact | Comfortable |
| UI | `visible_columns` | phone | any non-empty set of the results table columns | all |
| UI | `refresh_hz` | phone | 1-10 | 4 |
| UI | `auto_scroll_log` | phone | on / off | on |
| Session | `auto_reconnect` | phone | on / off | off |
| Session | `remember_key` | phone | on / off (off = key in memory only) | on |
| Session | `last_device_name` | phone | text | empty |
| Export | `export_format` | phone | JSON, CSV, Both | Both |
| Export | `include_log`, `include_scan_history` | phone | on / off | on |
| OTA | `expected_version`, `auto_check_on_connect` | phone | text (16), on / off | empty, off |
| Watchlist | pinned BSSIDs with label and rules | phone | see below | empty |
| Read only | `REQUEST_CONN_PARAMS`, `coex_debug`, BLE bonding, firmware version, git SHA, build time, partition layout, OTA key fingerprint | build time | | |

## Apply and Revert
One bar at the bottom shows `N unsaved changes` (device-side fields only). **Apply to device** sends one `SaveSettingsAll` and shows a
spinner until `SettingsSaved`. On success the new values become the confirmed baseline and the revision counter increases. On refusal the bar
shows the reason, the offending field is highlighted and **nothing the device confirmed changes**. **Revert** returns the editor to the last
values the device confirmed. A section tab shows a dot while it holds changes. Phone-side settings do not use the bar: they save at once and
show "saved locally".

## Cross-field and device rules (enforced by the core, mirrored in the app for instant feedback)
- A custom hop list holds channels 1-13 **once each** (0 = unused slot) and **at least one** channel. It is ignored unless the order is Custom.
- `deauth_window_ms` must be **at least twice `scan_interval_ms`**, otherwise `WindowTooShort`. Because the window tops out at 5000 ms, a scan
  interval above 2500 ms is always rejected.
- `default_channel` is 0 or 1-13; the channel filter may only set bits for channels 1-13.
- A refused save writes nothing: the core validates first, then persists, then adopts the values.

What each setting does: `scan_interval_ms` is the least time between two reports of the **same** network (new ones are never held back);
`dedup_rssi_db` is how far its RSSI must move before it is reported again; the SSID filter is a case-insensitive substring; `deauth_window_ms` is the
period the deauth counters are drained over, and the threshold is a rate per second. `hop_dwell_ms` and the hop order are handed to the radio **when
a scan starts**, never mid-hop. `log_level` sets the device's own (serial) logging; the device does not send log lines over Bluetooth.

## Sections

**Connection.** Link and key state (same as the top bar, which also has a lock icon that opens this section), the **device key** field (moved here
from its own panel) with Save and Copy, *remember key* and *reconnect automatically*, **Disconnect** (keeps the key), **Forget key** (two-step: clears the
saved key, drops the link, clears this session's scans and alerts) and **Rotate device key** (two-step, explanation only: rotating is a build-time action,
rebuild with a new key and reflash, other phones lose access; it is deliberately not possible over the wire).

**Scan, Radio, Telemetry, Alerts.** The device-side groups above. Radio's *default channel* and the Scan tab's channel buttons stay in step: picking a
channel in either pins the radio at once and shows in the other; Apply makes it permanent. Wire: `GetSettingsAll` / `SettingsAll`, `SaveSettingsAll` /
`SettingsSaved`.

**OTA.** Firmware version, git commit, build time and the **fingerprint of the OTA signing key** the firmware trusts (read only, copyable). It must equal
the value printed by `make fw-fingerprint` (also printed by `make fw-sign`). The phone can warn when the device runs a version other than the one you
expect. No key editing: signing keys are build time only. First-flash requirement: `docs/TESTING.md#signed-ota`. (The firmware update panel itself is below
the Settings panel.) Wire: `GetDeviceInfo` / `DeviceInfo`.

**Storage.** Read-only diagnostics loaded on demand with *Refresh*: NVS bytes used and free, the two OTA slots with the active one marked, and the
**settings revision counter** (it counts every save and survives reboots; an unchanged number after a power cycle proves the settings persisted).
**Erase NVS settings** (two-step) writes the factory defaults through the normal save path: it does not reboot and keeps the device key.
Wire: `GetStorageInfo` / `StorageInfo`.

**System.** Free heap and current channel (live from Status), when this phone connected (the firmware does not report uptime), **Reboot** (two-step),
**Factory reset** (two-step plus typing `RESET`: erases the settings **and the claimed key**, reboots the device unclaimed, and makes this phone forget the
key and device name; it is refused with `Rejected { Busy }` while an update is in flight), **Self-test** and the build-time facts (`REQUEST_CONN_PARAMS`,
`coex_debug`, BLE bonding "Not enabled"), greyed with a tooltip saying why they are read only. The self-test checks radio (tune to channel 1 then 13, restore),
NVS (write, read, erase a scratch key), Bluetooth (advertising or connected) and the update slot (ota_0 or ota_1 running); it answers `busy` without touching
the radio while a scan is running. Wire: `FactoryReset`, `RunSelfTest` / `SelfTestResult`, `Rejected`.

**UI, Watchlist, Import/Export (phone only).** UI: theme, signal unit (bars use the -30/-67/-70/-80/-90 bands), density, results columns, refresh cap, log
auto-scroll. Watchlist: BSSID, label, which events to alert on (appear, disappear, move by N dB) and on/off; add from the networks heard so far, by typing a
BSSID, or with the **PIN** button on a Scanner row; alerts go to the Alerts feed, the Log and the Status panel's "last event" line. Import/Export: export phone
settings plus the watchlist (and optionally the session log and scan history) as one JSON file with a schema version; **import shows a diff first and
rejects unknown keys**; export and import **device settings** as a separate file (importing loads it into the editor, Apply sends one `SaveSettingsAll`);
export the session as JSON, CSV or both.

## Compatibility
`SettingsAll` carries a schema number. If the firmware is older or newer than the app, or never answers, a banner says so and the device-side sections are
greyed out; phone-side sections keep working.

## Wire messages
Commands: `GetSettingsAll`, `SaveSettingsAll`, `FactoryReset`, `RunSelfTest`, `GetStorageInfo`, `GetDeviceInfo` (and the older `GetSettings` / `SetSettings`,
kept). Telemetry: `SettingsAll`, `SettingsSaved`, `StorageInfo`, `SelfTestResult`, `DeviceInfo`, `Rejected`. All appended; protocol version 1.
