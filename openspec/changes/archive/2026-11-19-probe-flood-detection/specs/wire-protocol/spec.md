# Spec Delta

## ADDED Requirements

### Requirement: Probe flood alert telemetry message
The wire protocol SHALL define `TelemetryMessage::ProbeFloodAlert { rate: u16, unique_clients: u16 }` as an appended variant, preserving postcard backwards compatibility.

#### Scenario: Encoding and decoding probe flood alert
- **WHEN** the device publishes `ProbeFloodAlert` with observed arrival rate and unique client MAC count
- **THEN** the message serializes to postcard and decodes into a matching JSON representation in the web client
