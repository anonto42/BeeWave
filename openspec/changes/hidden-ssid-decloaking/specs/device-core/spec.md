# Spec Delta: device-core

## ADDED Requirements

### Requirement: Hidden SSID automatic decloaking
The core scan table SHALL track whether a known BSSID was originally heard with an empty SSID, and when a subsequent frame provides a non-empty SSID for that BSSID, SHALL bypass deduplication thresholds and emit `TelemetryMessage::DecloakedResult`.

#### Scenario: Network heard initially with empty SSID
- **WHEN** an observation for a new BSSID has an empty SSID
- **THEN** it is inserted into `ScanTable` with `had_empty_ssid = true` and emitted as `ScanResult`

#### Scenario: Hidden network uncloaked by probe response
- **WHEN** a subsequent observation for that BSSID has a non-empty SSID, even if RSSI delta is under threshold or minimum interval has not elapsed
- **THEN** `ScanTable` updates the entry, marks `had_empty_ssid = false`, and returns `TelemetryMessage::DecloakedResult`

#### Scenario: Empty beacon does not overwrite known SSID
- **WHEN** an entry already has a known non-empty SSID and a subsequent observation arrives with an empty SSID
- **THEN** `ScanTable` does not revert `had_empty_ssid` to true and retains the known SSID status
