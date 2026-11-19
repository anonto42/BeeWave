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

### Requirement: BleFloodAlert telemetry message
The protocol crate SHALL define a `BleFloodAlert { rate: u16, class: AdClass }` variant on `TelemetryMessage` serialized via Postcard.

#### Scenario: Serialization round-trip
- **WHEN** a `BleFloodAlert` message is serialized to bytes via Postcard
- **THEN** it deserializes back into an identical struct matching golden wire test vectors

#### Scenario: Golden test vectors
- **WHEN** `cargo test -p beewave-protocol --features std --test vectors` runs
- **THEN** the `BleFloodAlert` test vector is verified and recorded in `web/test/vectors.json`

### Requirement: DisassocAlert telemetry message
The wire protocol SHALL define a `DisassocAlert { bssid: [u8; 6], count: u16 }` variant on `TelemetryMessage` encoded with Postcard and appended to existing variants, maintaining wire backwards compatibility.

#### Scenario: Serialization round-trip
- **WHEN** a `DisassocAlert` message is serialized to bytes via Postcard
- **THEN** it deserializes back into an identical struct matching golden test vectors

#### Scenario: Golden test vectors
- **WHEN** golden test vectors are regenerated
- **THEN** both Rust and TypeScript codecs encode and decode `DisassocAlert` identically

### Requirement: FindMy AdClass and BleStats wire representation
The wire protocol SHALL append `FindMy` to the `AdClass` enum and append `find_my: u16` to the `BleStats` telemetry message.

#### Scenario: AdClass deserialization
- **WHEN** an `AdClass::FindMy` is serialized and deserialized
- **THEN** it matches between Rust and TypeScript codecs

#### Scenario: BleStats with FindMy
- **WHEN** `BleStats` carrying `find_my` counts is encoded
- **THEN** the TypeScript codec decodes the field accurately

### Requirement: Provisioning messages
The protocol SHALL define a `Provision { key }` command (32 bytes) and an `Unclaimed` telemetry message, appended after existing variants.

#### Scenario: Golden vectors
- **WHEN** Rust encodes `Provision` and `Unclaimed`
- **THEN** the TypeScript codec decodes and encodes the same bytes, checked by golden vectors

### Requirement: Passive handshake capture messages
The protocol SHALL define `SetHandshakeCapture { enabled }` and `GetHandshakeCapture` commands and `HandshakeCaptureState { enabled }`, `HandshakeFrame { bssid, client, message, replay_counter, truncated, eapol }` and `HandshakeComplete { bssid, client }` telemetry, appended after existing variants with the version unchanged.

#### Scenario: Golden vectors
- **WHEN** Rust encodes each handshake message, including a truncated `HandshakeFrame`
- **THEN** the TypeScript codec decodes and re-encodes identical bytes, checked by golden vectors

### Requirement: Monitoring messages
The protocol SHALL define `GetMonitor` and `SetMonitor { deauth_threshold, ble_enabled, ble_spike_factor }` commands and `DeauthAlert { bssid, count }`, `BleStats`, `BleSpike`, `AnomalyAlert` and `MonitorConfig` telemetry, plus the `AdClass` and `AnomalyKind` enums, all appended after existing variants with the protocol version unchanged.

#### Scenario: Golden vectors
- **WHEN** Rust encodes each monitoring message
- **THEN** the TypeScript codec produces and decodes identical bytes, checked by golden vectors

### Requirement: Settings management messages
The protocol SHALL define `GetSettingsAll`, `SaveSettingsAll`, `FactoryReset`, `RunSelfTest`, `GetStorageInfo` and `GetDeviceInfo` commands and `SettingsAll`, `SettingsSaved`, `StorageInfo`, `SelfTestResult`, `DeviceInfo` and `Rejected` telemetry, appended after existing variants with the version unchanged, and SHALL keep `GetSettings` and `SetSettings`.

#### Scenario: Golden vectors
- **WHEN** Rust encodes each settings message
- **THEN** the TypeScript codec produces and decodes identical bytes, checked by golden vectors

### Requirement: SecurityDowngrade AnomalyKind wire representation
The wire protocol SHALL define `SecurityDowngrade` as a variant of the `AnomalyKind` enum, appended after existing variants to preserve wire backwards compatibility.

#### Scenario: AnomalyKind round-trip
- **WHEN** an `AnomalyAlert` carrying `AnomalyKind::SecurityDowngrade` is encoded with Postcard
- **THEN** it deserializes back identically across both Rust and TypeScript implementations matching golden vectors

### Requirement: Probe flood alert telemetry message
The wire protocol SHALL define `TelemetryMessage::ProbeFloodAlert { rate: u16, unique_clients: u16 }` as an appended variant, preserving postcard backwards compatibility.

#### Scenario: Encoding and decoding probe flood alert
- **WHEN** the device publishes `ProbeFloodAlert` with observed arrival rate and unique client MAC count
- **THEN** the message serializes to postcard and decodes into a matching JSON representation in the web client
