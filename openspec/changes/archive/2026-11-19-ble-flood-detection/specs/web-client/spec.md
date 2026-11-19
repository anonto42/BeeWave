# Spec Delta: web-client

## ADDED Requirements

### Requirement: BLE flood alert ingestion and display
The web client SHALL decode `BleFloodAlert` telemetry messages and publish a high-severity alert to the Alert feed and toast system.

#### Scenario: Display BLE flood alert
- **WHEN** a `BleFloodAlert` packet is decoded
- **THEN** an alert with source "bleFlood" and severity "alert" is added to `AlertStore` detailing the packet rate and dominant spam class
