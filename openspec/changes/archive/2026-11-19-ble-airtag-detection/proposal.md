# Proposal

## Why
Bluetooth Low Energy trackers, most notably Apple AirTags and Find My network accessories (such as Chipolo ONE Spot and Pebblebee), broadcast periodic BLE advertisements for offline finding. These devices can be abused for unwanted tracking and physical surveillance.

While BeeWave currently monitors BLE advertisements and classifies Google Fast Pair, Apple iBeacon, and AirPods Proximity pairing, it ignores Apple Find My offline finding payloads (Continuity type `0x12`), treating them as generic background traffic. Adding dedicated classification for Apple Find My / AirTag devices enables real-time tracker detection, presence monitoring, spike alerts, and anti-stalking protection.

## What Changes
- Add `FindMy` variant to `AdClass` enum in `crates/protocol`.
- Add `find_my: u16` field to `BleStats` telemetry message.
- Update `classify_structure` in `crates/core/src/features/ble_monitor/classify.rs` to identify Apple manufacturer data (`0x004C`) with Continuity sub-type `0x12` (`APPLE_FIND_MY`).
- Update `BleMonitor` and `BleFloodDetector` in `crates/core` to track 5 advertisement classes (`CLASSES = 5`).
- Regenerate wire protocol golden vectors (`make vectors`).
- In `web`, mirror `findMy` in `AD_CLASSES`, update `BleStats` decoder, and render Apple Find My (AirTag) in the BLE Monitor table, sparklines, spike warnings, and flood alerts.
- Add comprehensive unit tests in Rust and TypeScript covering Find My advertisement parsing, rolling statistics, and alerts.

## Non-goals and Invariants
- Receive-only operation; BeeWave transmits no BLE scan requests or connections.
- `#![no_std]` and zero-allocation execution in `crates/core`.
- Backward-compatible postcard wire encoding.

## Capabilities

### Modified Capabilities
- `device-core`: Recognize Apple Find My / AirTag advertisements, tracking their rolling counts, baselines, and spike alerts.
- `wire-protocol`: Add `FindMy` variant to `AdClass` and `find_my` to `BleStats` telemetry message.
- `web-client`: Ingest `findMy` counts in `BleStats`, displaying Find My trackers in the BLE Monitor panel and alert feed.

## Impact
- `crates/protocol`: Extends `AdClass` and `BleStats`.
- `crates/core`: Updates BLE classification logic and array capacities in `features/ble_monitor/`.
- `web`: Updates TypeScript protocol types, decoder, monitor state, and template labels.
