## ADDED Requirements

### Requirement: Signed OTA end message
The protocol SHALL define `OtaEndSigned` carrying the 64-byte Ed25519 signature as two 32-byte halves, appended after existing variants, so that its body is byte-identical to a raw openssl Ed25519 signature.

#### Scenario: Golden vector
- **WHEN** vectors are regenerated
- **THEN** the TypeScript encoder produces the same bytes for `OtaEndSigned`
