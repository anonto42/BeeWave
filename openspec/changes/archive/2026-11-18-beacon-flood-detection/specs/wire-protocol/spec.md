# Spec Delta: wire-protocol

## ADDED Requirements

### Requirement: BeaconFloodAlert telemetry message
The protocol crate SHALL define a `BeaconFloodAlert { rate: u16, unique_bssids: u16 }` variant on `TelemetryMessage` serialized via Postcard.

#### Scenario: Serialization round-trip
- **WHEN** a `BeaconFloodAlert` message is serialized to bytes via Postcard
- **THEN** it deserializes back into an identical struct matching golden wire test vectors

#### Scenario: Golden test vectors
- **WHEN** `cargo test -p beewave-protocol --features std --test vectors` runs
- **THEN** the `BeaconFloodAlert` test vector is verified and recorded in `web/test/vectors.json`
