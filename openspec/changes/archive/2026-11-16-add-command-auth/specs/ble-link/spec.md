## ADDED Requirements

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
