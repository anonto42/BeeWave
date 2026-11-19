# Proposal

## Why

KARMA and MANA attack suites (used in devices like the WiFi Pineapple or tools like `hostapd-mana` and `airbase-ng -P`) listen to client Probe Requests and dynamically broadcast beacons or responses pretending to be whichever network nearby devices are probing for. In many deployments, rogue AP tools reuse a single radio BSSID or rapidly switch their advertised SSID to capture multiple client connections. In standard 802.11 operation, a physical Access Point BSSID never changes its SSID identity on the fly. Currently, BeeWave detects when a known SSID appears on a new BSSID (`NewBssidForSsid`), but when an existing BSSID broadcasts a completely different SSID, `AnomalyDetector` silently overwrites the previous SSID hash without alerting. Adding KARMA / Rogue AP SSID mutation detection flags these honeypot attacks in real time.

## What Changes

- Add `KarmaSsidChange` variant to `AnomalyKind` in `crates/protocol/src/types.rs`.
- Regenerate golden test vectors in `crates/protocol` and `web/test/vectors.json`.
- In `crates/core/src/features/anomaly/`:
  - Increase `KINDS` from 5 to 6 in `types.rs`.
  - In `detector.rs`, detect when an existing non-empty SSID hash `e.ssid != 0` encounters a new non-empty `ssid != 0` with `e.ssid != ssid`, raising `AnomalyKind::KarmaSsidChange`.
  - Add unit tests verifying KARMA honeypot detection while ignoring hidden / cloaked networks (where SSID is 0).
- In `web/`:
  - Add `"karmaSsidChange"` to `ANOMALY_KINDS` in `web/src/shared/protocol/monitor.ts`.
  - Format `karmaSsidChange` in `web/src/shared/monitor/alerts.ts` with high severity (`alert` / red).
  - Simulate the anomaly in `web/src/shared/transport/mock-world.ts` and test with vitest.

## Capabilities

### Modified Capabilities

- `wire-protocol`: Add `KarmaSsidChange` variant to `AnomalyKind` enum.
- `device-core`: Detect Access Point BSSID SSID alteration anomalies in `#![no_std]`.
- `web-client`: Ingest and display `karmaSsidChange` anomaly alerts in the alert feed.

## Impact

- Wire protocol: Backwards-compatible variant appended to Postcard enum.
- Core: Zero heap allocation in `#![no_std]` core.
- Web: Extended anomaly kind list and alert feed text.
