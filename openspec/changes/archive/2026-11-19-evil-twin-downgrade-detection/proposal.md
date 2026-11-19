# Proposal

## Why
In wireless Rogue Access Point attacks (such as those executed via WiFi Pineapple, Wifiphisher, Airgeddon, or rogue AP scripts), attackers broadcast "Evil Twin" access points that clone legitimate network SSIDs while stripping encryption—downgrading from WPA2 or WPA3-Personal/Enterprise down to Open or legacy WEP—to trick client devices and users into connecting to a rogue captive portal.

While BeeWave currently detects when a known SSID appears on a new BSSID (`NewBssidForSsid`), it does not correlate the security suite of the new AP with the known baseline. As a result, an attacker broadcasting an unencrypted clone of a secure network is only flagged with a mild informational hint rather than an active rogue AP attack. Adding `SecurityDowngrade` anomaly detection enables real-time alerts whenever a secured network's SSID is cloned with Open or degraded security.

## What Changes
- Add `SecurityDowngrade` variant to `AnomalyKind` enum in `crates/protocol`.
- In `crates/core/src/features/anomaly/detector.rs`, record the security protocol (`WifiSecurityProtocol`) associated with observed SSIDs and detect when an SSID previously observed with WPA2, WPA3, or Enterprise encryption appears with Open or WEP security.
- Emit `AnomalyKind::SecurityDowngrade` with high priority.
- Regenerate wire protocol golden vectors (`make vectors`).
- In `web`, mirror `securityDowngrade` in `ANOMALY_KINDS` in `web/src/shared/protocol/monitor.ts` and format high-severity alerts in `web/src/shared/monitor/alerts.ts` (`[EVIL TWIN] Security downgrade: Open/unencrypted access point detected for secured network "<SSID>"`).
- Add unit tests in Rust host tests and vitest.

## Non-goals and Invariants
- BeeWave remains strictly receive-only.
- Memory remains bounded without dynamic heap allocations in `#![no_std]`.
- Non-breaking backwards-compatible postcard wire encoding.

## Capabilities

### Modified Capabilities
- `device-core`: Detect Evil Twin security downgrades where an SSID known to use WPA2/WPA3/Enterprise appears with Open or WEP security, emitting `AnomalyAlert`.
- `wire-protocol`: Add `SecurityDowngrade` variant to `AnomalyKind`.
- `web-client`: Ingest `securityDowngrade` anomalies and display high-severity Evil Twin alerts in the alert feed and notifications.

## Impact
- `crates/protocol`: Appends `SecurityDowngrade` to `AnomalyKind`.
- `crates/core`: Extends `AnomalyDetector` entry tracking and evaluation logic.
- `web`: Updates protocol kinds and alert formatting.
