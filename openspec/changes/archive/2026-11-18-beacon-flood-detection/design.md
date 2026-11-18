# Design

## Context
See `proposal.md` for motivation. Offensive firmware tools such as Hydra-ESP (`attack_beacon_spam.c`) broadcast bursts of beacon frames with spoofed BSSIDs to flood airtime and jam Wi-Fi scanners. BeeWave already features deauthentication flood detection and BLE advertisement spike detection; adding beacon flood detection completes the wireless flood detection suite.

## Goals / Non-Goals

**Goals:**
- Passive, zero-allocation detection of rapid beacon bursts with diverse BSSIDs.
- Distinction between a legitimate high-density AP environment and an active beacon spam attack (requiring both high frame velocity and multiple unique BSSIDs).
- Strict alert cooldown to protect BLE bandwidth during ongoing attacks.
- High-visibility red alert notification in the Web PWA.

**Non-Goals:**
- Active frame jamming or transmission.
- Channel jamming counter-measures.

## Decisions

### Decision 1: Wire protocol representation
Add `BeaconFloodAlert { rate: u16, unique_bssids: u16 }` to `TelemetryMessage` in `crates/protocol/src/types.rs`.
**Rationale**: Keeps wire serialization minimal (approx 6 bytes with Postcard varints) and matches the design pattern established by `DeauthAlert`.

### Decision 2: Zero-allocation tracking in `BeaconFloodDetector`
Implement `BeaconFloodDetector` in `crates/core/src/features/scan/flood.rs`:
- Tracks `current_beacons: u16` and up to 32 unique BSSIDs (`heapless::Vec<[u8; 6], 32>`) in the active 1-second window.
- On each beacon observation, increments `current_beacons` and adds the BSSID if not already present (up to capacity).
- On each 1-second `tick`, evaluates if `current_beacons >= 25` and `unique_bssids.len() >= 8`.
- If triggered and `cooldown == 0`, produces `BeaconFloodAlert` and resets `cooldown = 5`.
- Resets window counters every tick.
**Rationale**: Runs entirely in `#![no_std]` with bounded RAM (< 250 bytes) and requires zero heap allocation.

### Decision 3: Alert presentation in PWA
The alert is surfaced with severity `alert` (red) under the alert source `beaconFlood`.

## Risks / Trade-offs

- **[High-density commercial environments (e.g. conferences)]** → Requiring both high beacon rate (25+/s) on a single channel and a burst of unique new BSSIDs prevents false positives from normal multi-SSID enterprise APs.
- **[BLE link saturation during flood]** → Cooldown timer limits alerts to at most 1 every 5 seconds.
