## ADDED Requirements

### Requirement: Monitoring messages
The protocol SHALL define `GetMonitor` and `SetMonitor { deauth_threshold, ble_enabled, ble_spike_factor }` commands and `DeauthAlert { bssid, count }`, `BleStats`, `BleSpike`, `AnomalyAlert` and `MonitorConfig` telemetry, plus the `AdClass` and `AnomalyKind` enums, all appended after existing variants with the protocol version unchanged.

#### Scenario: Golden vectors
- **WHEN** Rust encodes each monitoring message
- **THEN** the TypeScript codec produces and decodes identical bytes, checked by golden vectors
