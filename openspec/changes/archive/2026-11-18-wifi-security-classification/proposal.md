# Proposal

## Why
Wi-Fi networks employ diverse security configurations—ranging from insecure Open and legacy WEP/WPA networks to modern WPA2-PSK, WPA3-SAE, Opportunistic Wireless Encryption (OWE), and 802.1X Enterprise. Furthermore, IEEE 802.11w Protected Management Frames (PMF) determines whether management frames (such as deauthentications) can be forged. Currently, BeeWave only parses SSID and channel from beacons and probe responses, leaving users and defenders blind to the security posture and PMF capabilities of surrounding Access Points.

## What Changes
- Add `SecurityResult` variant to `TelemetryMessage` in `crates/protocol` carrying BSSID, protocol (`Open`, `Wep`, `Wpa`, `Wpa2`, `Wpa3`, `Wpa2Wpa3`, `Owe`, `Enterprise`), PMF mode (`None`, `Capable`, `Required`), AKM suite, and pairwise cipher.
- Implement zero-allocation 802.11 Capability Information and RSN (Tag 48) / WPA (Tag 221) element parser in `crates/core/src/features/scan/security.rs`.
- Track security observations in `SecurityTracker` to deduplicate reports and emit updates only on discovery or security posture changes.
- Update golden wire vectors (`make vectors`) and mirror decoder in `web/src/shared/protocol/`.
- Update `ScanStore` to store security metadata and render security badges (`WPA3`, `WPA2`, `OPEN`, `PMF`, etc.) in Scanner and Results views.

## Capabilities

### Modified Capabilities
- `device-core`: Parse 802.11 capability flags, RSN elements (Tag 48), and WPA elements (Tag 221) zero-allocation to determine security protocol and PMF status, deduplicating through `SecurityTracker`.
- `wire-protocol`: Add `SecurityResult` variant to `TelemetryMessage` and update golden test vectors.
- `web-client`: Ingest `SecurityResult` telemetry into `ScanStore` and display security badges and PMF indicators in Scanner and Target panels.

## Impact
- `crates/protocol`: Non-breaking addition of `SecurityResult` variant to `TelemetryMessage`.
- `crates/core`: New `features/scan/security.rs` parser and `SecurityTracker` bounded deduplication table.
- `web`: New types, decoder support in `web/src/shared/protocol/`, scan store aggregation, and UI badges.
