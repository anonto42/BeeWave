# Spec Delta

## ADDED Requirements

### Requirement: Passive probe request flood detection
The core monitoring engine SHALL track received 802.11 Probe Request frame velocity and distinct client MAC counts in a 1-second sliding window without heap allocation, emitting `ProbeFloodAlert` when the rate and client diversity exceed configured thresholds.

#### Scenario: Normal probe arrival below threshold
- **WHEN** client probe requests arrive at normal ambient rates (below 30 frames per second)
- **THEN** no `ProbeFloodAlert` is emitted

#### Scenario: Probe request flood attack exceeds threshold
- **WHEN** probe requests arrive at 30 or more frames per second with at least 8 unique client MAC addresses in a 1-second window
- **THEN** the core emits `ProbeFloodAlert` containing the observed rate and unique client count

#### Scenario: Flood alert cooldown rate limiting
- **WHEN** a probe flood attack persists across consecutive 1-second intervals
- **THEN** the core rate-limits `ProbeFloodAlert` emissions to at most one alert every 5 seconds to prevent BLE link exhaustion
