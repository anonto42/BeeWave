# wire-protocol Specification Delta

## ADDED Requirements

### Requirement: Wi-Fi PHY Standard Classification
The protocol crate SHALL define a `WifiPhyStandard` enum representing physical layer standards (`Unknown`, `B`, `G`, `N`, `Ax`) and append a `PhyResult` variant to `TelemetryMessage` carrying the Access Point BSSID and its detected PHY standard.

#### Scenario: Golden vector verification
- **WHEN** `TelemetryMessage::PhyResult` is serialized and deserialized via Postcard
- **THEN** both Rust and TypeScript codecs encode and decode the identical binary representation
