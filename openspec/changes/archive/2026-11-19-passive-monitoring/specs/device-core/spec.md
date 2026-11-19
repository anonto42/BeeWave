## ADDED Requirements

### Requirement: Receive-only deauthentication detector
The core SHALL count deauthentication frames per BSSID over a one second window and SHALL raise a `DeauthAlert` for each BSSID at or above the configured threshold, and MUST NOT transmit anything.

#### Scenario: Threshold crossed
- **WHEN** a BSSID reaches the threshold within one second
- **THEN** a `DeauthAlert` with that BSSID and count is published

#### Scenario: Below threshold or disabled
- **WHEN** the count is below the threshold, or the threshold is 0
- **THEN** no alert is published

### Requirement: Receive-only BLE advertisement monitor
The core SHALL classify received advertisements as Fast Pair, iBeacon, proximity pairing or generic, SHALL keep per-class counts for the last minute, and SHALL raise `BleSpike` when a class exceeds its baseline by the configured factor after a warm-up.

#### Scenario: Classification
- **WHEN** an advertisement carries Apple manufacturer data of type 0x02 and length 0x15
- **THEN** it is classified as iBeacon

#### Scenario: Spike
- **WHEN** a class count exceeds its baseline times the factor after warm-up
- **THEN** a `BleSpike` naming the class is published

### Requirement: Anomaly alerts
The core SHALL raise `AnomalyAlert` for a known SSID on a new BSSID, a BSSID seen on two channels within a minute, an RSSI jump above 20 dB between cycles, and a BSSID returning after more than 5 missed cycles, with a cooldown per BSSID and kind.

#### Scenario: Cooldown
- **WHEN** the same anomaly repeats within the cooldown
- **THEN** it is not published again

### Requirement: Monitor configuration
The core SHALL validate `SetMonitor`, answer with `MonitorConfig`, and reject out-of-range values with an event.

#### Scenario: Invalid factor
- **WHEN** `ble_spike_factor` is outside 2..=20
- **THEN** the configuration is unchanged and an error event is published
