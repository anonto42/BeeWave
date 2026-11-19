## ADDED Requirements

### Requirement: Validated persistent device settings
The core SHALL validate every device-side setting against its range and the cross-field rules, SHALL persist a complete valid set and bump a revision counter, and MUST leave storage and live values untouched when a request is rejected.

#### Scenario: Out of range
- **WHEN** a field is outside its range
- **THEN** `SettingsSaved` reports `ok: false` with the reason naming the field and nothing is written

#### Scenario: Window too short
- **WHEN** `deauth_window_ms` is less than twice `scan_interval_ms`
- **THEN** the request is rejected with `WindowTooShort`

#### Scenario: Revision
- **WHEN** a save succeeds
- **THEN** the revision increases by one and survives a reboot

### Requirement: Settings take effect where they belong
The core SHALL apply scan filters and the dedup threshold to following beacons, SHALL hand hop dwell and order to the radio only when a scan starts, and SHALL use the saved status period and deauth window.

#### Scenario: Hop change mid-scan
- **WHEN** hop settings are saved while a scan is running
- **THEN** the radio keeps its current hopping until the next scan start

### Requirement: Factory reset
The core SHALL accept `FactoryReset` only from an authenticated session with no update in flight, erase settings, key and revision, and reboot.

#### Scenario: Update in flight
- **WHEN** `FactoryReset` arrives during an update
- **THEN** it is rejected with `Busy` and nothing is erased

### Requirement: Self-test
The core SHALL run a self-test of radio, NVS, BLE and OTA slot when not scanning and report each result, and MUST return `busy` without touching the radio while scanning.

#### Scenario: Scanning
- **WHEN** `RunSelfTest` arrives during a scan
- **THEN** the result has `busy: true` and no radio call is made
