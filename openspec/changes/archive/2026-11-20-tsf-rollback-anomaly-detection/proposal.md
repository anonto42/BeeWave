# Proposal: Beacon Timestamp (TSF) Inversion & Rogue AP Anomaly Detection

## Why

Genuine physical Wi-Fi Access Points maintain internal hardware clocks that increment the 64-bit microsecond Timing Synchronization Function (TSF) monotonically from boot ($1\mu\text{s}$ per tick). Software rogue AP tools (such as `airbase-ng`, `wifiphisher`, `mdk4`, or ESP32 Marauder beacon spoofers) often initialize TSF timestamps to 0 or emit erratic, nonsensical clock values.

When an attacker launches a rogue AP or clone of an existing network with a spoofed BSSID, the observed TSF timestamp rolls back significantly compared to the genuine AP's uptime. Detecting when an AP's timestamp jumps backward by 5+ seconds provides high-confidence attribution of active rogue AP / fake AP impersonation attacks.

## What Changes

- **Protocol**: Add `TsfRollback` variant to `AnomalyKind` enum in `crates/protocol/src/types.rs`.
- **Core**:
  - Extract the 64-bit little-endian TSF timestamp (bytes 24..32) in `crates/core/src/features/scan/frame.rs` and store `tsf: u64` in `Observation`.
  - In `crates/core/src/features/anomaly/detector.rs`, track `last_tsf: u64` per BSSID. If an existing BSSID is observed with a timestamp that rolls back by $\ge 5$ seconds ($5,000,000\,\mu\text{s}$), emit `AnomalyKind::TsfRollback` with `detail` containing the rollback magnitude in seconds (capped at 255).
- **Web Client**:
  - Add `"tsfRollback"` to `ANOMALY_KINDS` in `web/src/shared/protocol/monitor.ts`.
  - Format high-severity alert in `web/src/shared/monitor/alerts.ts`: `[ROGUE AP DETECTED] Timestamp rollback on <bssid> (<detail>s backward) - possible fake AP`.
  - Add simulation in `MockWorld` and test cases.

## Capabilities

### Modified Capabilities
- `wire-protocol`: Add `TsfRollback` variant to `AnomalyKind`.
- `device-core`: Parse beacon TSF timestamp and detect backward timestamp rollbacks.
- `web-client`: Ingest and display rogue AP timestamp rollback alerts.

## Impact

- `crates/protocol`: Extends `AnomalyKind`. Test vectors updated via `make vectors`.
- `crates/core`: Zero heap allocation; 8 bytes added to `Entry` and `Observation`.
- `web`: New high-severity alert formatted and displayed in the alert feed.
