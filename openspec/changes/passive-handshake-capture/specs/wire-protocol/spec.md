## ADDED Requirements

### Requirement: Passive handshake capture messages
The protocol SHALL define `SetHandshakeCapture { enabled }` and `GetHandshakeCapture` commands and `HandshakeCaptureState { enabled }`, `HandshakeFrame { bssid, client, message, replay_counter, truncated, eapol }` and `HandshakeComplete { bssid, client }` telemetry, appended after existing variants with the version unchanged.

#### Scenario: Golden vectors
- **WHEN** Rust encodes each handshake message, including a truncated `HandshakeFrame`
- **THEN** the TypeScript codec decodes and re-encodes identical bytes, checked by golden vectors
