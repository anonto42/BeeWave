# wire-protocol Specification

## Purpose

Defines the compact binary packets exchanged between the ESP32 and the PWA over BLE GATT, identically encoded by Rust and TypeScript.

## Requirements

### Requirement: Versioned sequenced packets
Every packet SHALL be `[version u8][sequence u8][postcard body]` and SHALL fit in one ATT payload (negotiated MTU >= 64).

#### Scenario: Unknown version
- **WHEN** a packet arrives with a version other than the supported one
- **THEN** the receiver rejects it without acting on its body

#### Scenario: Truncated packet
- **WHEN** a packet is shorter than its declared fields
- **THEN** decoding fails with an error and no partial message is produced

### Requirement: Byte-identical codecs
The TypeScript codec MUST produce and consume exactly the bytes of the Rust codec for every message.

#### Scenario: Golden vectors
- **WHEN** `make vectors` regenerates `web/test/vectors.json`
- **THEN** `make web-test` passes, proving both implementations agree

### Requirement: Loss detection
Receivers SHALL detect lost telemetry from sequence gaps modulo 256.

#### Scenario: Gap
- **WHEN** sequence numbers skip values
- **THEN** the lost counter increases by the number of skipped packets
