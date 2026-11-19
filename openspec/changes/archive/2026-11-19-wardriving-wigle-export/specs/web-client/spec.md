# Spec Delta: web-client

## ADDED Requirements

### Requirement: WiGLE.net wardriving CSV export
The web client SHALL support acquiring geolocation coordinates, associating them with scanned networks, and generating standard WiGLE v1.4 compatible CSV files for wardriving.

#### Scenario: WiGLE CSV header and formatting
- **WHEN** networks are exported in WiGLE format
- **THEN** the output includes the `WigleWifi-1.4` metadata header, standard column headers, and data rows with correctly calculated frequencies and security capability strings

#### Scenario: Network coordinates attachment
- **WHEN** the GPS tracker is active and a network is observed
- **THEN** the network record and scan log store the current latitude, longitude, and accuracy

#### Scenario: Export without active GPS
- **WHEN** networks are exported while GPS is inactive
- **THEN** coordinates default to 0.0 with 0.0 accuracy without crashing
