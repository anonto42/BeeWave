# Spec Delta: web-client

## ADDED Requirements

### Requirement: Security telemetry decoding and storage
The web client SHALL decode `SecurityResult` messages received over Web Bluetooth and attach security and PMF information to the corresponding `NetworkRecord` in `ScanStore`.

#### Scenario: Telemetry message dispatch
- **WHEN** a `SecurityResult` telemetry packet is decoded
- **THEN** `ScanStore.ingestSecurity` updates the target BSSID record and notifies change listeners

### Requirement: Security badges in scanner and target views
The web client SHALL display security badges (such as `OPEN`, `WEP`, `WPA2`, `WPA3`, `OWE`, and `PMF`) on Access Point rows in Scanner and Target views.

#### Scenario: Display security badge
- **WHEN** an Access Point record has associated security information
- **THEN** the Scanner row displays a badge identifying the protocol and PMF status
