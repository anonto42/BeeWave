# Spec Delta: web-client

## ADDED Requirements

### Requirement: Decloaked result ingestion and display
The web client SHALL decode `DecloakedResult` telemetry messages, update the network record in `ScanStore` with `decloaked: true`, post an informational alert to `AlertStore`, and render a `DECLOAKED` badge in the scanner view.

#### Scenario: Display decloaked badge
- **WHEN** a network has `decloaked: true` in `ScanStore`
- **THEN** the scanner row renders a `DECLOAKED` badge alongside the uncloaked SSID

#### Scenario: Alert emitted on decloak
- **WHEN** a `DecloakedResult` telemetry message is received
- **THEN** an alert is recorded in `AlertStore` noting the revealed SSID and channel
