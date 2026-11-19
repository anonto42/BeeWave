# Proposal

## Why
Offensive wireless penetration testing devices (such as Hydra-ESP with `attack_bt_spam.c`, Flipper Zero, and ESP32 Marauder) execute Bluetooth Low Energy (BLE) spam attacks by broadcasting bursts of spoofed advertisement packets. These attacks trigger relentless pairing modals, battery status popups, and setup notifications on nearby iOS devices (Apple Action Modals / AirPods popups), Android phones (Google FastPair), and Windows machines (SwiftPair).

While BeeWave passively classifies BLE advertisements and computes 60-second rolling statistics (`BleStats`) and baseline spikes (`BleSpike`), it cannot detect sudden high-velocity packet floods in real time (e.g., 30+ packets/second). Detecting active BLE spam requires an immediate 1-second sliding-window flood detector that identifies the flood rate and the dominant spam class without waiting for multi-minute baseline adaptation.

## What Changes
- Add `BleFloodAlert { rate: u16, class: AdClass }` variant to `TelemetryMessage` in `crates/protocol`.
- Implement zero-allocation `BleFloodDetector` in `crates/core/src/features/ble_monitor/flood.rs` tracking packet arrival velocity and identifying the dominant advertisement class over a 1-second sliding window.
- Integrate `BleFloodDetector` into the monitoring pipeline on `on_ble_adv` and 1 Hz `on_tick`, rate-limiting alert emissions (5-second cooldown) to protect BLE transport bandwidth.
- Regenerate golden wire test vectors (`make vectors`) and mirror the decoder in `web/src/shared/protocol/`.
- Ingest `BleFloodAlert` in the web client's `AlertStore` and display high-severity alert entries (`[BLE FLOOD ATTACK] Rapid BLE advertisement flood: <rate>/s (<class> spam)`).
- Add mock BLE flood simulation in `web/src/shared/transport/mock-world.ts` and add unit tests across protocol, core, and web.

## Capabilities

### Modified Capabilities
- `device-core`: Detect BLE advertisement flooding attacks by tracking arrival velocity per second, identifying the dominant class, and emitting `BleFloodAlert`.
- `wire-protocol`: Add `BleFloodAlert` variant to `TelemetryMessage` and update golden test vectors.
- `web-client`: Ingest `BleFloodAlert` telemetry into `AlertStore` and present high-severity alerts in the feed and notifications.

## Impact
- `crates/protocol`: Appends `BleFloodAlert` variant to `TelemetryMessage` (non-breaking).
- `crates/core`: Adds `features/ble_monitor/flood.rs` with `BleFloodDetector` and wires it into `Monitoring`.
- `web`: Updates protocol decoder, adds alert formatting for BLE floods, and adds mock simulation in `mock-world.ts`.
