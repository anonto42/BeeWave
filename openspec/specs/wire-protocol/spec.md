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

### Requirement: Challenge-response authentication messages
The protocol SHALL define `AuthBegin` and `AuthProve { mac }` commands and `AuthChallenge { nonce }`
and `AuthResult { ok }` telemetry, appended after existing variants, where the proof is
HMAC-SHA256 over the ASCII prefix `beewave-auth-v1` followed by the 16-byte nonce.

#### Scenario: Cross-language proof
- **WHEN** Rust and TypeScript compute the proof for the same key and nonce
- **THEN** both produce identical bytes, checked by a golden vector

#### Scenario: Replayed proof
- **WHEN** a proof for an earlier nonce is sent after a new `AuthBegin`
- **THEN** the device rejects it

### Requirement: Signed OTA end message
The protocol SHALL define `OtaEndSigned` carrying the 64-byte Ed25519 signature as two 32-byte halves, appended after existing variants, so that its body is byte-identical to a raw openssl Ed25519 signature.

#### Scenario: Golden vector
- **WHEN** vectors are regenerated
- **THEN** the TypeScript encoder produces the same bytes for `OtaEndSigned`

### Requirement: Passive PMKID telemetry message
The protocol SHALL define `PmkidFound { bssid: [u8; 6], client: [u8; 6], pmkid: [u8; 16] }` appended to `TelemetryMessage` with the wire protocol version unchanged.

#### Scenario: Golden vectors
- **WHEN** Rust encodes a `PmkidFound` telemetry message
- **THEN** the TypeScript codec decodes and re-encodes identical bytes, checked by golden vectors

### Requirement: Client probe telemetry message
The protocol SHALL define `ProbeResult { client: [u8; 6], channel: u8, rssi: i8, ssid: String<32> }` appended to `TelemetryMessage` with the wire protocol version unchanged.

#### Scenario: Golden vectors
- **WHEN** Rust encodes a `ProbeResult` telemetry message
- **THEN** the TypeScript codec decodes and re-encodes identical bytes, checked by golden vectors

### Requirement: AssocResult Telemetry Message
The wire protocol SHALL define an `AssocResult` variant in `TelemetryMessage` encoded with postcard, carrying the Access Point BSSID (`[u8; 6]`), client station MAC (`[u8; 6]`), channel (`u8`), and RSSI (`i8`).

#### Scenario: Serializing and deserializing AssocResult
- **WHEN** an `AssocResult` telemetry message is encoded with postcard
- **THEN** it deserializes identically on both device core and web client with matching golden vectors

### Requirement: WpsResult Telemetry Message
The wire protocol SHALL define a `WpsResult` variant in `TelemetryMessage` encoded with postcard, carrying the Access Point BSSID (`[u8; 6]`), WPS version (`u8`), setup state (`u8`), locked status (`bool`), selected registrar status (`bool`), device name (`String<32>`), manufacturer (`String<32>`), and model name (`String<32>`).

#### Scenario: Serializing and deserializing WpsResult
- **WHEN** a `WpsResult` telemetry message is encoded with postcard
- **THEN** it deserializes identically on both device core and web client with matching golden vectors

### Requirement: SecurityResult telemetry message
The protocol crate SHALL define a `SecurityResult` telemetry message variant serialized via Postcard carrying the BSSID, security protocol enum/tag, PMF mode, AKM suite, and pairwise cipher.

#### Scenario: Serialization round-trip
- **WHEN** a `SecurityResult` message is serialized to bytes via Postcard
- **THEN** it deserializes back into an identical struct matching golden wire test vectors

#### Scenario: Golden test vectors
- **WHEN** `cargo test -p beewave-protocol --features std --test vectors` runs
- **THEN** the `SecurityResult` test vector is verified and recorded in `web/test/vectors.json`

### Requirement: BeaconFloodAlert telemetry message
The protocol crate SHALL define a `BeaconFloodAlert { rate: u16, unique_bssids: u16 }` variant on `TelemetryMessage` serialized via Postcard.

#### Scenario: Serialization round-trip
- **WHEN** a `BeaconFloodAlert` message is serialized to bytes via Postcard
- **THEN** it deserializes back into an identical struct matching golden wire test vectors

#### Scenario: Golden test vectors
- **WHEN** `cargo test -p beewave-protocol --features std --test vectors` runs
- **THEN** the `BeaconFloodAlert` test vector is verified and recorded in `web/test/vectors.json`

### Requirement: DecloakedResult telemetry message
The protocol crate SHALL define a `DecloakedResult { bssid: [u8; 6], ssid: String<32>, channel: u8, rssi: i8 }` variant on `TelemetryMessage` serialized via Postcard.

#### Scenario: Serialization round-trip
- **WHEN** a `DecloakedResult` message is serialized to bytes via Postcard
- **THEN** it deserializes back into an identical struct matching golden wire test vectors

#### Scenario: Golden test vectors
- **WHEN** `cargo test -p beewave-protocol --features std --test vectors` runs
- **THEN** the `DecloakedResult` test vector is verified and recorded in `web/test/vectors.json`
