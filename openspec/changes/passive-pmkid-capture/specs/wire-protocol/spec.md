# Spec Delta

## ADDED Requirements

### Requirement: Passive PMKID telemetry message
The protocol SHALL define `PmkidFound { bssid: [u8; 6], client: [u8; 6], pmkid: [u8; 16] }` appended to `TelemetryMessage` with the wire protocol version unchanged.

#### Scenario: Golden vectors
- **WHEN** Rust encodes a `PmkidFound` telemetry message
- **THEN** the TypeScript codec decodes and re-encodes identical bytes, checked by golden vectors
