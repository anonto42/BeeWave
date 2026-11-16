# device-core Specification

## Purpose

Hardware-independent device behaviour (radio control, scan reporting, status, OTA) implemented in `crates/core` behind ports.

## Requirements

### Requirement: Validated radio control
The radio feature SHALL accept channels 1 through 13 and MUST reject any other channel without calling the driver.

#### Scenario: Invalid channel
- **WHEN** `ConfigureRadio` carries channel 99
- **THEN** the command fails with an invalid-channel error and the current channel is unchanged

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
