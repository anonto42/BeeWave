# web-client Specification Delta

## ADDED Requirements

### Requirement: Rogue AP TSF Rollback Alert Formatting
The web client SHALL decode `TsfRollback` anomaly alerts and render high-severity alert messages in the alert feed noting the observed timestamp rollback magnitude in seconds.

#### Scenario: Display rogue AP alert
- **WHEN** an anomaly alert with kind `tsfRollback` and detail `N` is received
- **THEN** the alert feed displays `[ROGUE AP DETECTED] Timestamp rollback on <bssid> (<N>s backward) - possible fake AP` with alert severity
