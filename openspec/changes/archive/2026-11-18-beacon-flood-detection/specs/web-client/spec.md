# Spec Delta: web-client

## ADDED Requirements

### Requirement: Beacon flood alert ingestion and display
The web client SHALL decode `BeaconFloodAlert` telemetry messages and publish a high-severity alert to the Alert feed and toast system.

#### Scenario: Display beacon flood alert
- **WHEN** a `BeaconFloodAlert` packet is decoded
- **THEN** an alert with source "beaconFlood" and severity "alert" is added to `AlertStore` detailing the frame rate and unique BSSID count
