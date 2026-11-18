# Proposal

## Why
Wireless attack tools such as Hydra-ESP (`attack_beacon_spam.c`), ESP32 Marauder, and mdk4 flood the 2.4 GHz band with dozens or hundreds of forged 802.11 beacon frames with randomized BSSIDs and SSIDs. This attack overwhelms client device scan lists, drains receiver resources, and degrades wireless monitoring accuracy. While BeeWave actively alerts on deauthentication floods (`DeauthAlert`) and BLE advertisement spikes (`BleSpike`), it currently has no mechanism to detect or alert on beacon flooding attacks.

## What Changes
- Add `BeaconFloodAlert { rate: u16, unique_bssids: u16 }` variant to `TelemetryMessage` in `crates/protocol`.
- Implement zero-allocation `BeaconFloodDetector` in `crates/core/src/features/scan/flood.rs` tracking beacon arrival velocity and unique BSSIDs per second, rate-limiting alert emissions to protect BLE bandwidth.
- Integrate `BeaconFloodDetector` into the monitoring pipeline and 1 Hz tick processing in `crates/core`.
- Update golden wire vectors (`make vectors`) and mirror decoder in `web/src/shared/protocol/`.
- Ingest `BeaconFloodAlert` in the web client's alert store and display critical red alerts in the Alerts feed and toast notifications.

## Capabilities

### Modified Capabilities
- `device-core`: Detect beacon flooding attacks by tracking beacon rate and unique BSSID counts per second, publishing `BeaconFloodAlert` when thresholds are exceeded.
- `wire-protocol`: Add `BeaconFloodAlert` variant to `TelemetryMessage` and update golden test vectors.
- `web-client`: Ingest `BeaconFloodAlert` telemetry into `AlertStore` and present high-severity alerts in the feed and notifications.

## Impact
- `crates/protocol`: Appends `BeaconFloodAlert` variant to `TelemetryMessage` (non-breaking).
- `crates/core`: Adds `features/scan/flood.rs` with `BeaconFloodDetector` and wires it into `Monitoring`.
- `web`: Updates protocol decoder, adds alert formatting for beacon floods, and adds mock simulation in `mock-world.ts`.
