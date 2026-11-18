# Spec Delta

## ADDED Requirements

### Requirement: Passive probe request parsing
The core SHALL parse 802.11 management frames of subtype `0x40` (Probe Request) without transmitting any response, extracting client transmitter MAC address, requested SSID (empty if wildcard/broadcast), channel, and RSSI.

#### Scenario: Directed probe request
- **WHEN** a client transmits a Probe Request with a non-empty SSID element
- **THEN** the core extracts the client MAC and requested SSID

#### Scenario: Wildcard probe request
- **WHEN** a client transmits a Probe Request with a 0-length SSID element
- **THEN** the core extracts the client MAC with an empty SSID string

### Requirement: Probe request deduplication
The core SHALL maintain a bounded cache of recently seen client probes and SHALL suppress duplicate telemetry unless the client probes for a new SSID or the signal moves by at least the configured deduplication threshold.

#### Scenario: Repeated probe suppression
- **WHEN** the same client probes repeatedly for the same SSID without a significant RSSI change
- **THEN** redundant probe telemetry is suppressed
