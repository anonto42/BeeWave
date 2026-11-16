# ble-link Specification

## Purpose

The BLE GATT link between phone and device: access control and delivery guarantees.

## Current state (not yet hardened)

Pairing is **not currently required**: the command characteristic accepts writes from any
connected central, unauthenticated. This was deliberately turned off because Android's "pair new
device" screen did not reliably bond with this plain custom GATT peripheral, and Web Bluetooth has
no way to drive that pairing UI either — an encrypted characteristic just hung with no visible
error on either side. The passkey-pairing code is written and kept in
`firmware/src/adapters/ble_gatt/pairing.rs` (`require()`), just not called. Do not expose this
device to anyone other than its operator until that requirement below is actually enforced again.

## Requirements

### Requirement: Authenticated command channel (deferred)
Once re-enabled, the device SHALL accept commands only over an encrypted, authenticated, bonded
link established with a passkey, and MUST NOT accept command writes from unpaired peers.

#### Scenario: Unpaired write
- **WHEN** an unpaired central writes the command characteristic
- **THEN** the write is rejected by the stack and no command is processed

### Requirement: Configurable passkey
The passkey SHALL be set at build time through `WALPAD_PASSKEY`, and the firmware MUST log a warning when it falls back to the public default.

#### Scenario: Default passkey
- **WHEN** `WALPAD_PASSKEY` is not set at build time
- **THEN** the default passkey is used and a warning is logged at startup

### Requirement: Visible write failures
A command write that the device rejects for any reason SHALL produce a response the client can see as an error, never a silent no-op.

#### Scenario: Rejected write
- **WHEN** the device refuses a command write
- **THEN** the client's send promise rejects and the failure is logged, instead of the command disappearing with no trace

### Requirement: Lossy telemetry, reliable commands path
Telemetry notifications SHALL never block the device, and the client SHALL expose lost-packet counts derived from sequence numbers.

#### Scenario: Slow link
- **WHEN** the telemetry queue is full
- **THEN** newer messages are dropped, the device keeps running, and the drop count appears in status
