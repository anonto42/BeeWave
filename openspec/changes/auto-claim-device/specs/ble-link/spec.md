## MODIFIED Requirements

### Requirement: Authenticated command session
The device SHALL reject every command except the authentication handshake and first-time provisioning until the client has proved knowledge of the shared key in the current connection, SHALL end the session whenever a connection is made or lost, and MUST stay locked once claimed to any other key.

#### Scenario: Unauthenticated command
- **WHEN** a client sends a command other than the handshake or provisioning before authenticating
- **THEN** the command is not executed and an unauthorized event is returned

#### Scenario: Session ends on disconnect
- **WHEN** an authenticated client disconnects and another connects
- **THEN** the new client must authenticate before any command is accepted

#### Scenario: Unclaimed device
- **WHEN** the device has no key and a client begins authentication
- **THEN** the device reports it is unclaimed and accepts one `Provision`, after which only that key authenticates

#### Scenario: No key configured
- **WHEN** the firmware was built without a build-time key and nothing is stored
- **THEN** the device starts unclaimed and the first client may claim it

#### Scenario: Claimed device
- **WHEN** a client sends `Provision` to a device that already has a key
- **THEN** the device refuses and its key is unchanged
