# Spec Delta: wire-protocol

## ADDED Requirements

### Requirement: DecloakedResult telemetry message
The protocol crate SHALL define a `DecloakedResult { bssid: [u8; 6], ssid: String<32>, channel: u8, rssi: i8 }` variant on `TelemetryMessage` serialized via Postcard.

#### Scenario: Serialization round-trip
- **WHEN** a `DecloakedResult` message is serialized to bytes via Postcard
- **THEN** it deserializes back into an identical struct matching golden wire test vectors

#### Scenario: Golden test vectors
- **WHEN** `cargo test -p beewave-protocol --features std --test vectors` runs
- **THEN** the `DecloakedResult` test vector is verified and recorded in `web/test/vectors.json`
