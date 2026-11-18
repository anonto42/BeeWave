# Spec Delta: device-core

## ADDED Requirements

### Requirement: Passive beacon flood detection
The core monitoring engine SHALL track beacon frame arrival velocity and unique BSSID counts per 1-second interval without heap allocation and emit a `BeaconFloodAlert` when the rate exceeds the flood threshold.

#### Scenario: Normal beacon reception below threshold
- **WHEN** beacon frames arrive at normal rates (below 25 frames per second)
- **THEN** no `BeaconFloodAlert` is emitted

#### Scenario: Beacon flood attack exceeds threshold
- **WHEN** beacon frames arrive exceeding 25 frames per second with at least 10 unique BSSIDs in a 1-second window
- **THEN** the core emits a `BeaconFloodAlert` carrying the rate and unique BSSID count

#### Scenario: Flood alert cooldown rate limiting
- **WHEN** a beacon flood continues continuously over multiple consecutive seconds
- **THEN** the core rate-limits `BeaconFloodAlert` emissions to at most one alert every 5 seconds to prevent BLE saturation
