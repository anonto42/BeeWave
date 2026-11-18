# Spec Delta

## ADDED Requirements

### Requirement: Discovered client devices view
The client SHALL display discovered client devices, showing client MAC, vendor resolution (distinguishing locally administered/randomized MACs), RSSI, channel, and their list of requested SSIDs.

#### Scenario: Probe telemetry received
- **WHEN** `ProbeResult` telemetry is received
- **THEN** the client device table updates with the client MAC, signal strength, and probed network name
