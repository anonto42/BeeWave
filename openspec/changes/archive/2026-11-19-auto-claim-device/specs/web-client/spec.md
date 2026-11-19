## MODIFIED Requirements

### Requirement: Device key and automatic authentication
The client SHALL authenticate automatically after connecting, SHALL claim an unclaimed device by generating a random key and saving it only after the device accepts it, SHALL let the user enter a key by hand for an already claimed device, and SHALL keep the key in browser storage without failing when storage is unavailable.

#### Scenario: Key stored
- **WHEN** the user saves a valid 64 hex character key and connects to a claimed device
- **THEN** the client completes the handshake without further input

#### Scenario: First connect
- **WHEN** the device reports it is unclaimed
- **THEN** the client provisions a new key, saves it after acceptance and the session is authenticated

#### Scenario: Claimed by someone else
- **WHEN** the device is claimed and the browser has no key
- **THEN** the UI shows that a key is needed

#### Scenario: Wrong key
- **WHEN** the device rejects the proof
- **THEN** the UI shows that authentication failed
