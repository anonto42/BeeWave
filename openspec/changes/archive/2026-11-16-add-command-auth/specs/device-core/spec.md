## ADDED Requirements

### Requirement: Authentication gate
The core SHALL verify proofs in constant time, SHALL use each nonce once, and MUST drop commands and non-auth telemetry while unauthenticated.

#### Scenario: Wrong proof
- **WHEN** the proof does not match
- **THEN** the result is not ok and the session stays unauthenticated

#### Scenario: Locked gate
- **WHEN** the gate has no key
- **THEN** every proof fails
