# Spec Delta: wire-protocol

## ADDED Requirements

### Requirement: SecurityResult telemetry message
The protocol crate SHALL define a `SecurityResult` telemetry message variant serialized via Postcard carrying the BSSID, security protocol enum/tag, PMF mode, AKM suite, and pairwise cipher.

#### Scenario: Serialization round-trip
- **WHEN** a `SecurityResult` message is serialized to bytes via Postcard
- **THEN** it deserializes back into an identical struct matching golden wire test vectors

#### Scenario: Golden test vectors
- **WHEN** `cargo test -p beewave-protocol --features std --test vectors` runs
- **THEN** the `SecurityResult` test vector is verified and recorded in `web/test/vectors.json`
