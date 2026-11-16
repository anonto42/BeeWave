## ADDED Requirements

### Requirement: Signed firmware updates
The OTA feature SHALL finish an update only after an Ed25519 signature over the announced image SHA-256 verifies against the built-in public key, MUST abort the update on a missing or invalid signature, and MUST refuse updates entirely when no public key is configured.

#### Scenario: Bad signature
- **WHEN** the signature does not verify
- **THEN** the update is aborted, the image is never finalised, and the client gets a bad-signature error

#### Scenario: Unsigned end
- **WHEN** the client ends an update without a signature
- **THEN** the update is aborted and refused as unsigned

#### Scenario: No signing key
- **WHEN** the device has no OTA public key
- **THEN** beginning an update is refused
