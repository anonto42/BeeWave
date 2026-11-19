# Spec Delta

## ADDED Requirements

### Requirement: FindMy AdClass and BleStats wire representation
The wire protocol SHALL append `FindMy` to the `AdClass` enum and append `find_my: u16` to the `BleStats` telemetry message.

#### Scenario: AdClass deserialization
- **WHEN** an `AdClass::FindMy` is serialized and deserialized
- **THEN** it matches between Rust and TypeScript codecs

#### Scenario: BleStats with FindMy
- **WHEN** `BleStats` carrying `find_my` counts is encoded
- **THEN** the TypeScript codec decodes the field accurately
