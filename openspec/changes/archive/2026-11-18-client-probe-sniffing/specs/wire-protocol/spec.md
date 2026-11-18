# Spec Delta

## ADDED Requirements

### Requirement: Client probe telemetry message
The protocol SHALL define `ProbeResult { client: [u8; 6], channel: u8, rssi: i8, ssid: String<32> }` appended to `TelemetryMessage` with the wire protocol version unchanged.

#### Scenario: Golden vectors
- **WHEN** Rust encodes a `ProbeResult` telemetry message
- **THEN** the TypeScript codec decodes and re-encodes identical bytes, checked by golden vectors
