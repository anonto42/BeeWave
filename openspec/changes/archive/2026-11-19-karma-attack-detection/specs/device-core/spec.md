# Spec Delta

## ADDED Requirements

### Requirement: KARMA rogue AP SSID change anomaly detection
The core anomaly detector SHALL detect when an Access Point BSSID previously seen advertising a valid non-empty SSID begins broadcasting a different non-empty SSID, emitting an `AnomalyAlert` of kind `KarmaSsidChange`.

#### Scenario: Rogue AP switches advertised SSID on same BSSID
- **WHEN** an AP with a known non-empty SSID is observed broadcasting a different non-empty SSID
- **THEN** the core emits `AnomalyAlert` of kind `KarmaSsidChange` naming the offending BSSID

#### Scenario: Hidden network transition ignored
- **WHEN** an AP broadcasts an empty or hidden SSID
- **THEN** no `KarmaSsidChange` anomaly is raised
