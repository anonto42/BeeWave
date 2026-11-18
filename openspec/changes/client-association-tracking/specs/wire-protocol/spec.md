# Spec Delta

## ADDED Requirements

### Requirement: AssocResult Telemetry Message
The wire protocol SHALL define an `AssocResult` variant in `TelemetryMessage` encoded with postcard, carrying the Access Point BSSID (`[u8; 6]`), client station MAC (`[u8; 6]`), channel (`u8`), and RSSI (`i8`).

#### Scenario: Serializing and deserializing AssocResult
- **WHEN** an `AssocResult` telemetry message is encoded with postcard
- **THEN** it deserializes identically on both device core and web client with matching golden vectors
