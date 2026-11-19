## ADDED Requirements

### Requirement: Provisioning messages
The protocol SHALL define a `Provision { key }` command (32 bytes) and an `Unclaimed` telemetry message, appended after existing variants.

#### Scenario: Golden vectors
- **WHEN** Rust encodes `Provision` and `Unclaimed`
- **THEN** the TypeScript codec decodes and encodes the same bytes, checked by golden vectors
