# Design

## Context
See `proposal.md` for motivation. Currently, `crates/core/src/features/ble_monitor/classify.rs` checks for `AD_MANUFACTURER` (type `0xFF`) with Apple Company ID `0x004C`, recognizing iBeacon (type `0x02`, length `0x15`) and Proximity Pairing (AirPods, type `0x07`).
Apple Find My offline finding advertisements (used by AirTags and certified third-party Find My tags) use Continuity sub-type `0x12` (`APPLE_FIND_MY`).

## Goals / Non-Goals

**Goals:**
- Classify Apple Find My / AirTag advertisements zero-copy and zero-allocation in `#![no_std]`.
- Track Find My rolling counts and baseline spikes in `BleMonitor`.
- Identify AirTag spam in `BleFloodDetector`.
- Render live counts and sparklines in the PWA BLE Monitor panel.

**Non-Goals:**
- Reverse-engineering the encrypted rotating public key payload.
- Sending BLE connection requests or pairing requests to detected tags.

## Decisions

### Decision 1: Add `FindMy` to `AdClass` and extend `BleStats`
- *Choice*: Add `FindMy` variant to `AdClass` and `find_my: u16` to `BleStats`.
- *Rationale*: Find My devices represent a distinct security and anti-stalking category separate from generic ambient beacons or audio pairing sheets.
- *RAM impact*: `BleCounts` array increases from 4 to 5 `u16`s (2 extra bytes per minute window). Zero dynamic allocations.

### Decision 2: Payload classification signature
- *Choice*: Check `data[0..2] == [0x4C, 0x00]` (Apple), `data[2] == 0x12` (`APPLE_FIND_MY`), and `data[3] >= 0x12` (minimum payload length).
- *Rationale*: Accurately matches standard Apple Find My offline finding advertisement payloads broadcast by AirTags.

## Risks / Trade-offs

- [Risk: Golden vectors mismatch] $\to$ Regenerate golden vectors via `make vectors` and update vitest test cases.
