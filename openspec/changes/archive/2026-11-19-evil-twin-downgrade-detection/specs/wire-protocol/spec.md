# Spec Delta

## ADDED Requirements

### Requirement: SecurityDowngrade AnomalyKind wire representation
The wire protocol SHALL define `SecurityDowngrade` as a variant of the `AnomalyKind` enum, appended after existing variants to preserve wire backwards compatibility.

#### Scenario: AnomalyKind round-trip
- **WHEN** an `AnomalyAlert` carrying `AnomalyKind::SecurityDowngrade` is encoded with Postcard
- **THEN** it deserializes back identically across both Rust and TypeScript implementations matching golden vectors
