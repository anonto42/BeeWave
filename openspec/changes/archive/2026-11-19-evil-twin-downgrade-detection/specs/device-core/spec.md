# Spec Delta

## ADDED Requirements

### Requirement: Evil Twin security downgrade anomaly detection
The core SHALL correlate the security protocol of observed access points against the known security baseline of matching SSIDs and SHALL raise `AnomalyAlert` with kind `SecurityDowngrade` when an SSID known to use robust encryption (WPA2, WPA3, Enterprise) is observed with Open or WEP security.

#### Scenario: Open rogue AP clones WPA2 network
- **WHEN** an access point is observed broadcasting an SSID previously heard with WPA2 or WPA3 encryption, but advertising Open or WEP security
- **THEN** the core emits `AnomalyAlert` with kind `SecurityDowngrade` naming the rogue BSSID

#### Scenario: Legitimate access points with matching security
- **WHEN** a new access point is observed for a known SSID with identical or compatible security posture
- **THEN** no `SecurityDowngrade` anomaly is emitted
