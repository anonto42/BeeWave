## ADDED Requirements

### Requirement: Challenge-response authentication messages
The protocol SHALL define `AuthBegin` and `AuthProve { mac }` commands and `AuthChallenge { nonce }`
and `AuthResult { ok }` telemetry, appended after existing variants, where the proof is
HMAC-SHA256 over the ASCII prefix `beewave-auth-v1` followed by the 16-byte nonce.

#### Scenario: Cross-language proof
- **WHEN** Rust and TypeScript compute the proof for the same key and nonce
- **THEN** both produce identical bytes, checked by a golden vector

#### Scenario: Replayed proof
- **WHEN** a proof for an earlier nonce is sent after a new `AuthBegin`
- **THEN** the device rejects it
