# Proposal

## Why
Access Points broadcast rich device metadata (manufacturer, model name, model number, device name) and Wi-Fi Protected Setup (WPS) security configuration (version, setup state, lockout status, selected registrar) inside 802.11 Vendor Specific Information Elements (`OUI 00:50:f2:04`). Currently, BeeWave only parses SSID and channel from beacons, missing valuable passive reconnaissance data regarding hardware models and WPS vulnerability posture (e.g., unlocked WPS PIN interfaces or active PushButton pairing).

## What Changes
- Add `WpsResult` variant to `TelemetryMessage` in `crates/protocol` carrying BSSID, WPS version, state, locked status, selected registrar, device name, manufacturer, and model name.
- Add zero-allocation WPS Information Element TLV parser in `crates/core/src/features/scan/wps.rs`.
- Track and deduplicate WPS observations in `crates/core` so static device metadata is transmitted efficiently without saturating the BLE link.
- Update golden wire vectors (`make vectors`) and mirror decoder in `web/src/shared/protocol/`.
- Display WPS version, lockout status, and hardware model in the Access Point target and scanner views.

## Capabilities

### Modified Capabilities
- `device-core`: Parse WPS vendor information elements from 802.11 Beacon and Probe Response frames into structured device metadata without allocation.
- `wire-protocol`: Add `WpsResult` variant to `TelemetryMessage`.
- `web-client`: Ingest `WpsResult` telemetry and display device model, manufacturer, and WPS security badges in Access Point views.

## Impact
- `crates/protocol`: Appends `WpsResult` variant to `TelemetryMessage` (non-breaking, golden vectors regenerated).
- `crates/core`: Extends `features/scan/` with `wps.rs` TLV parser and deduplication table.
- `firmware`: Promiscuous callback feeds beacon frames to existing scan pipeline which now also extracts WPS.
- `web`: `web/src/shared/protocol/` decodes `WpsResult`, `ScanStore` / target views associate WPS metadata with Access Points.
