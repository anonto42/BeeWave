# Spec Delta

## ADDED Requirements

### Requirement: Evil Twin security downgrade alert presentation
The web client SHALL decode `SecurityDowngrade` anomaly alerts and render them in the alert feed with alert-level severity (red), identifying the rogue BSSID and explaining that an unencrypted clone of a secured network was detected.

#### Scenario: Display Evil Twin alert
- **WHEN** an `AnomalyAlert` packet with kind `securityDowngrade` is decoded
- **THEN** an alert is posted in `AlertStore` with severity `alert` and title `Evil Twin security downgrade detected`
