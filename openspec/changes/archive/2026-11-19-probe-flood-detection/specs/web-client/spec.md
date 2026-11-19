# Spec Delta

## ADDED Requirements

### Requirement: Probe flood alert ingestion and presentation
The web client SHALL decode `ProbeFloodAlert` telemetry messages and display them in the Alert feed and toast system with alert-level severity (red), indicating the observed probe frame rate and unique client MAC count.

#### Scenario: Ingesting probe flood alert
- **WHEN** the web client receives a `probeFloodAlert` telemetry message
- **THEN** it formats an alert with title indicating rapid probe request flood, detail displaying the rate and client count, and severity `alert`
