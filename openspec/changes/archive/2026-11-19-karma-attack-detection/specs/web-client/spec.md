# Spec Delta

## ADDED Requirements

### Requirement: KARMA SSID change alert presentation
The web client SHALL decode `KarmaSsidChange` anomaly alerts and render them in the alert feed with alert-level severity (red), identifying the BSSID and explaining that an active KARMA/MANA rogue AP or SSID honeypot mutation was detected.

#### Scenario: Ingesting KARMA anomaly alert
- **WHEN** the web client receives an `anomaly` telemetry message with kind `karmaSsidChange`
- **THEN** it formats an alert with title "KARMA / Rogue AP SSID change detected" and severity `alert`
