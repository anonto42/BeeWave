# ble-link Specification

## Purpose

The BLE GATT link between phone and device: access control and delivery guarantees.

## Current state

Commands are protected by an application-level HMAC challenge-response (see the *Authenticated
command session* requirement). The link itself is **not encrypted** and BLE bonding is still off:
Android's "pair new device" screen did not reliably bond with this plain custom GATT peripheral and
Web Bluetooth cannot drive that UI. The passkey code is kept in
`firmware/src/adapters/ble_gatt/pairing.rs` (`require()`), unwired. Bonding remains a possible
future hardening step, tracked by the deferred requirement below.

## Requirements

### Requirement: Authenticated command channel (deferred)
Once re-enabled, the device SHALL accept commands only over an encrypted, authenticated, bonded
link established with a passkey, and MUST NOT accept command writes from unpaired peers.

#### Scenario: Unpaired write
- **WHEN** an unpaired central writes the command characteristic
- **THEN** the write is rejected by the stack and no command is processed

### Requirement: Configurable passkey
The passkey SHALL be set at build time through `BEEWAVE_PASSKEY`, and the firmware MUST log a warning when it falls back to the public default.

#### Scenario: Default passkey
- **WHEN** `BEEWAVE_PASSKEY` is not set at build time
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

### Requirement: Authenticated command session
The device SHALL reject every command except the authentication handshake until the client has
proved knowledge of the shared key in the current connection, SHALL end the session whenever a
connection is made or lost, and MUST stay locked when no valid key is configured.

#### Scenario: Unauthenticated command
- **WHEN** a client sends a command other than the handshake before authenticating
- **THEN** the command is not executed and an unauthorized event is returned

#### Scenario: Session ends on disconnect
- **WHEN** an authenticated client disconnects and another connects
- **THEN** the new client must authenticate before any command is accepted

#### Scenario: No key configured
- **WHEN** the firmware was built without a valid `BEEWAVE_KEY`
- **THEN** no client can authenticate and all commands are rejected

### Requirement: Telemetry gated by authentication
Until the session is authenticated the device SHALL publish only authentication messages and
unauthorized events, and SHALL NOT publish status or scan results.

#### Scenario: Unauthenticated listener
- **WHEN** a client connects but does not authenticate
- **THEN** it receives no status or scan telemetry
