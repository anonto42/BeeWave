# Proposal

## Why

Wireless penetration testing tools and attack suites (such as `mdk4 p`, `mdk3 p`, or ESP32 Marauder probe spam) broadcast massive surges of 802.11 Probe Request frames (`subtype 0x40`) with randomized MAC addresses and fake SSIDs. These attacks aim to overwhelm Access Point client tracking tables, crash wireless firmware, or blind monitoring tools. While BeeWave detects Beacon flooding (`BeaconFloodAlert`), Disassociation flooding (`DisassocAlert`), Deauthentication flooding (`DeauthAlert`), and Bluetooth Low Energy advertising floods (`BleFloodAlert`), it currently lacks detection for Probe Request floods. Unthrottled probe surges fill queues and generate excessive individual `ProbeResult` messages. Adding rate-based probe flood detection provides real-time alerting and protection for the monitoring pipeline.

## What Changes

- Add `ProbeFloodAlert { rate: u16, unique_clients: u16 }` to `TelemetryMessage` in `crates/protocol/src/types.rs`.
- Regenerate wire protocol golden test vectors in `crates/protocol` and `web/test/vectors.json`.
- Implement `ProbeFloodDetector` in `crates/core/src/features/probe/flood.rs` with sliding 1-second velocity tracking, unique client MAC counting (capped without heap allocation), and a 5-second cooldown.
- Integrate `ProbeFloodDetector` into `Monitoring` in `crates/core/src/app/monitoring.rs`.
- Decode `probeFloodAlert` in `web/src/shared/protocol/` and format alerts in `web/src/shared/monitor/alerts.ts` with high severity (`alert` / red).
- Simulate probe flood alerts in `web/src/shared/transport/mock-world.ts` for end-to-end testing.

## Capabilities

### Modified Capabilities

- `wire-protocol`: Add `ProbeFloodAlert` variant to `TelemetryMessage`.
- `device-core`: Add passive 802.11 probe request flood detection in `#![no_std]`.
- `web-client`: Ingest and display `probeFloodAlert` messages in the alert feed with high severity.

## Impact

- Wire protocol: Backward-compatible variant addition to `TelemetryMessage` (Postcard enum tag appended).
- Core: Zero heap allocation in `#![no_std]` core.
- Web: Extended decoder and alert feed representation.
