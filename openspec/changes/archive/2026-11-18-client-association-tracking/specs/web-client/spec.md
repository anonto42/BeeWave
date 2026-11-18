# Spec Delta

## ADDED Requirements

### Requirement: Connected Stations Display and Association Correlation
The web client SHALL ingest `AssocResult` telemetry messages into an association store, mapping connected client stations to Access Point targets. The web client SHALL display connected station counts and station details (MAC address, vendor OUI resolution, RSSI, and last active timestamp) in the Access Point target views.

#### Scenario: Ingesting association telemetry
- **WHEN** an `AssocResult` telemetry message is received for an Access Point BSSID
- **THEN** the client station is added to or updated in the Access Point's connected station list

#### Scenario: Displaying connected stations under target
- **WHEN** an Access Point card in the Scan tab is expanded
- **THEN** all client stations associated with that Access Point are listed with resolved vendor and signal strength
