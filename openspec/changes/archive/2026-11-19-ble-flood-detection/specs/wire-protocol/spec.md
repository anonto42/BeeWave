# Spec Delta: wire-protocol

## ADDED Requirements

### Requirement: BleFloodAlert telemetry message
The protocol crate SHALL define a `BleFloodAlert { rate: u16, class: AdClass }` variant on `TelemetryMessage` serialized via Postcard.

#### Scenario: Serialization round-trip
- **WHEN** a `BleFloodAlert` message is serialized to bytes via Postcard
- **THEN** it deserializes back into an identical struct matching golden wire test vectors

#### Scenario: Golden test vectors
- **WHEN** `cargo test -p beewave-protocol --features std --test vectors` runs
- **THEN** the `BleFloodAlert` test vector is verified and recorded in `web/test/vectors.json`
