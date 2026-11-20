# device-core Specification Delta

## ADDED Requirements

### Requirement: Beacon TSF Extraction and Rollback Detection
The scan feature SHALL extract the 64-bit microsecond TSF timestamp from beacon management frames, and the anomaly detector SHALL flag an `AnomalyKind::TsfRollback` alert when an Access Point's timestamp rolls backward by 5 seconds or more relative to its previous observation.

#### Scenario: Rogue AP timestamp rollback
- **WHEN** an Access Point's beacon is received with a TSF timestamp that is at least 5,000,000 microseconds less than the previously observed timestamp for that BSSID
- **THEN** the anomaly detector raises `AnomalyKind::TsfRollback` with detail set to the rollback duration in seconds

#### Scenario: Normal monotonic clock progression
- **WHEN** an Access Point's beacon is received with an incremented or equal TSF timestamp
- **THEN** no `TsfRollback` anomaly is raised
