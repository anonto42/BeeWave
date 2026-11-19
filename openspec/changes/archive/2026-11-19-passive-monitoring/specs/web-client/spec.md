## ADDED Requirements

### Requirement: Passive monitoring panels
The client SHALL provide an expanded results table, channel occupancy and utilisation, RSSI history with a delta view, deauth, BLE and anomaly panels, scan comparison with JSON export, a phone-local watchlist, and a session export in JSON and CSV, all built from telemetry already received and without sending radio-affecting commands.

#### Scenario: Delta view
- **WHEN** a network's RSSI moves by at least 6 dB since the previous cycle
- **THEN** it is highlighted in the delta view

#### Scenario: Session export
- **WHEN** the user exports the session
- **THEN** a JSON and a CSV file containing scans, alerts, log and authentication events are downloaded

#### Scenario: Watchlist
- **WHEN** a pinned BSSID appears, disappears or moves by at least 6 dB
- **THEN** an alert is raised and nothing is stored on the device
