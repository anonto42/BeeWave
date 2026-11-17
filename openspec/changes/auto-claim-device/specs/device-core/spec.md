## MODIFIED Requirements

### Requirement: Authentication gate
The core SHALL verify proofs in constant time, SHALL use each nonce once, SHALL accept a key from a client only while it has none (persisting it before claiming), and MUST drop commands and non-auth telemetry while unauthenticated.

#### Scenario: Wrong proof
- **WHEN** the proof does not match
- **THEN** the result is not ok and the session stays unauthenticated

#### Scenario: Claim once
- **WHEN** an unclaimed device receives a valid `Provision`
- **THEN** the key is stored, the session is authenticated and a second `Provision` is refused

#### Scenario: Storage failure
- **WHEN** the key cannot be stored
- **THEN** the device stays unclaimed and reports failure

#### Scenario: Locked gate
- **WHEN** the gate has no key
- **THEN** every proof fails
