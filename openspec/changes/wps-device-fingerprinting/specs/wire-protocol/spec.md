# Spec Delta

## ADDED Requirements

### Requirement: WpsResult Telemetry Message
The wire protocol SHALL define a `WpsResult` variant in `TelemetryMessage` encoded with postcard, carrying the Access Point BSSID (`[u8; 6]`), WPS version (`u8`), setup state (`u8`), locked status (`bool`), selected registrar status (`bool`), device name (`String<32>`), manufacturer (`String<32>`), and model name (`String<32>`).

#### Scenario: Serializing and deserializing WpsResult
- **WHEN** a `WpsResult` telemetry message is encoded with postcard
- **THEN** it deserializes identically on both device core and web client with matching golden vectors
