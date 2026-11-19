# Spec Delta

## ADDED Requirements

### Requirement: Apple Find My tracker visualization
The web client SHALL decode `BleStats` with `findMy` counts, display Apple Find My (AirTag) in the BLE Monitor table with rolling sparklines, and format tracker spike warnings in the alert feed.

#### Scenario: Display Find My counts
- **WHEN** `BleStats` telemetry is received
- **THEN** the BLE Monitor table displays active counts and history for Apple Find My

#### Scenario: Find My spike alert
- **WHEN** a `BleSpike` for `findMy` is received
- **THEN** an alert is posted in `AlertStore` naming Apple Find My
