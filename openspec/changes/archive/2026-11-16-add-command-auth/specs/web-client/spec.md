## ADDED Requirements

### Requirement: Device key and automatic authentication
The client SHALL let the user enter the device key, keep it in browser storage without failing when storage is unavailable, and authenticate automatically after connecting.

#### Scenario: Key stored
- **WHEN** the user saves a valid 64 hex character key and connects
- **THEN** the client completes the handshake without further input

#### Scenario: Wrong key
- **WHEN** the device rejects the proof
- **THEN** the UI shows that authentication failed
