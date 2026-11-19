# Spec Delta

## ADDED Requirements

### Requirement: DisassocAlert telemetry message
The wire protocol SHALL define a `DisassocAlert { bssid: [u8; 6], count: u16 }` variant on `TelemetryMessage` encoded with Postcard and appended to existing variants, maintaining wire backwards compatibility.

#### Scenario: Serialization round-trip
- **WHEN** a `DisassocAlert` message is serialized to bytes via Postcard
- **THEN** it deserializes back into an identical struct matching golden test vectors

#### Scenario: Golden test vectors
- **WHEN** golden test vectors are regenerated
- **THEN** both Rust and TypeScript codecs encode and decode `DisassocAlert` identically
