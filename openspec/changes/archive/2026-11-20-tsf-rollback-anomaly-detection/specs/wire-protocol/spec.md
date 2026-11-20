# wire-protocol Specification Delta

## ADDED Requirements

### Requirement: TSF Rollback Anomaly Kind
The protocol crate SHALL include a `TsfRollback` variant in the `AnomalyKind` enum, appended after existing variants, to represent backward beacon timestamp anomalies indicative of rogue Access Points.

#### Scenario: Golden vector verification
- **WHEN** an `AnomalyAlert` with kind `TsfRollback` is serialized to Postcard
- **THEN** Rust and TypeScript codecs encode and decode identical byte representations
