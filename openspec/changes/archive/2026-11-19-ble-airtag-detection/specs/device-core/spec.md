# Spec Delta

## ADDED Requirements

### Requirement: Apple Find My and AirTag BLE classification
The core SHALL inspect Bluetooth Low Energy advertisement manufacturer data for Apple Company ID (`0x004C`) with Continuity sub-type `0x12` (`APPLE_FIND_MY`), classify matching packets as `AdClass::FindMy`, track their rolling counts across 1-minute windows, and evaluate spike conditions.

#### Scenario: AirTag advertisement received
- **WHEN** a BLE advertisement with Apple manufacturer data type 0x12 is observed
- **THEN** it is classified as `FindMy` and credited to the Find My class counter

#### Scenario: Find My spike alert
- **WHEN** Find My advertisement counts exceed the baseline by the configured spike factor after warm-up
- **THEN** a `BleSpike` naming `AdClass::FindMy` is published
