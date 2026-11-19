# Spec Delta: device-core

## ADDED Requirements

### Requirement: Real-time BLE flood detection
The core monitoring engine SHALL track BLE advertising arrival rates per 1-second interval without heap allocation and emit a `BleFloodAlert` carrying the rate and dominant advertisement class when the rate exceeds the flood threshold.

#### Scenario: Normal BLE advertisement rate below threshold
- **WHEN** BLE advertisement frames arrive at normal ambient rates (below 30 packets per second)
- **THEN** no `BleFloodAlert` is emitted

#### Scenario: BLE spam attack exceeds threshold
- **WHEN** BLE advertisement frames arrive exceeding 30 packets per second in a 1-second window
- **THEN** the core emits a `BleFloodAlert` carrying the rate and dominant advertisement class

#### Scenario: Flood alert cooldown rate limiting
- **WHEN** a BLE flood attack continues continuously over multiple consecutive seconds
- **THEN** the core rate-limits `BleFloodAlert` emissions to at most one alert every 5 seconds to prevent BLE saturation
