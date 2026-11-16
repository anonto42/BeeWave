# device-core Specification

## Purpose

Hardware-independent device behaviour (radio control, scan reporting, status, OTA) implemented in `crates/core` behind ports.

## Requirements

### Requirement: Validated radio control
The radio feature SHALL accept channels 1 through 13, or 0 meaning automatic hopping, and MUST reject any other channel without calling the driver. Status SHALL report the channel the radio is actually on.

#### Scenario: Invalid channel
- **WHEN** `ConfigureRadio` carries channel 99
- **THEN** the command fails with an invalid-channel error and the current channel is unchanged

### Requirement: Beacon parsing
The scan feature SHALL parse 802.11 beacon and probe-response frames into a BSSID, SSID, channel and RSSI without allocating, and MUST ignore other frame types and malformed or truncated frames without panicking.

#### Scenario: Hidden network
- **WHEN** a beacon carries an empty or all-zero SSID
- **THEN** the observation has an empty SSID

#### Scenario: Truncated element
- **WHEN** an information element claims more bytes than the frame holds
- **THEN** parsing stops and returns what was read so far

### Requirement: De-duplicated scan reporting
The scan feature SHALL report a network only when it is new or its RSSI moved by at least 6 dB, and SHALL bound its table without allocating.

#### Scenario: Stable network
- **WHEN** the same BSSID is observed again within 6 dB
- **THEN** no telemetry is emitted

#### Scenario: Table full
- **WHEN** more distinct networks are seen than the table can hold
- **THEN** the table evicts and continues reporting without failing

### Requirement: Sequential acknowledged OTA
The OTA feature SHALL accept chunks only at the expected offset, SHALL bound total size to the announced size, and MUST NOT finish unless all bytes were received.

#### Scenario: Out-of-order chunk
- **WHEN** a chunk arrives at the wrong offset
- **THEN** it is not written and an acknowledgement carries the expected offset

#### Scenario: Chunk without begin
- **WHEN** a chunk arrives with no active session
- **THEN** it is rejected as a bad-state error

### Requirement: Lossy telemetry
The core SHALL never block on the telemetry sink and SHALL count dropped messages and report the count in status.

#### Scenario: Sink full
- **WHEN** the sink rejects messages
- **THEN** the dropped counter increases and the app continues

### Requirement: Persistent settings
The settings feature SHALL validate the channel (0 for auto, or 1 through 13), SHALL save settings through the storage port before accepting them, and MUST keep the previous settings when storage fails. Saved settings SHALL be applied at startup.

#### Scenario: Invalid channel
- **WHEN** `SetSettings` carries channel 99
- **THEN** it is rejected, nothing is saved and an error event is emitted

#### Scenario: Storage failure
- **WHEN** the storage port fails to save
- **THEN** the in-memory settings are unchanged and an error event is emitted

#### Scenario: Boot
- **WHEN** the device starts with saved settings
- **THEN** the saved channel is applied and scanning starts if scan-at-boot was saved

### Requirement: Authentication gate
The core SHALL verify proofs in constant time, SHALL use each nonce once, and MUST drop commands and non-auth telemetry while unauthenticated.

#### Scenario: Wrong proof
- **WHEN** the proof does not match
- **THEN** the result is not ok and the session stays unauthenticated

#### Scenario: Locked gate
- **WHEN** the gate has no key
- **THEN** every proof fails
