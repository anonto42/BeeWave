# ble-link Specification

## Purpose

The BLE GATT link between phone and device: pairing, access control and delivery guarantees.

## Requirements

### Requirement: Authenticated command channel
The device SHALL accept commands only over an encrypted, authenticated, bonded link established with a passkey, and MUST NOT accept command writes from unpaired peers.

#### Scenario: Unpaired write
- **WHEN** an unpaired central writes the command characteristic
- **THEN** the write is rejected by the stack and no command is processed

### Requirement: Configurable passkey
The passkey SHALL be set at build time through `WALPAD_PASSKEY`, and the firmware MUST log a warning when it falls back to the public default.

#### Scenario: Default passkey
- **WHEN** `WALPAD_PASSKEY` is not set at build time
- **THEN** the default passkey is used and a warning is logged at startup

### Requirement: Lossy telemetry, reliable commands path
Telemetry notifications SHALL never block the device, and the client SHALL expose lost-packet counts derived from sequence numbers.

#### Scenario: Slow link
- **WHEN** the telemetry queue is full
- **THEN** newer messages are dropped, the device keeps running, and the drop count appears in status
