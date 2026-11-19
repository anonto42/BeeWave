# Spec Delta

## ADDED Requirements

### Requirement: Disassociation alert ingestion and presentation
The web client SHALL decode `DisassocAlert` telemetry messages, add them to `AlertStore`, and display them in the Alert feed and toast system with the source BSSID and observed frame rate.

#### Scenario: Ingesting disassociation alert
- **WHEN** a `DisassocAlert` packet is received and decoded
- **THEN** an alert is recorded in `AlertStore` with source `disassoc` and formatted with the BSSID and rate

#### Scenario: High severity threshold
- **WHEN** the disassociation count reaches or exceeds 50 frames per second
- **THEN** the alert severity is elevated to alert level (red)
