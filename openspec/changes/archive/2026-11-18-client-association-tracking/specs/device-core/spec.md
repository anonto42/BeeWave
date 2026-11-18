# Spec Delta

## ADDED Requirements

### Requirement: Passive Client-to-AP Association Tracking
The core SHALL parse 802.11 Data and QoS Data frame headers to extract client station MAC and Access Point BSSID addresses from ToDS/FromDS flags without heap allocation, ignoring broadcast/multicast and invalid addresses. The core SHALL track associations in a bounded table, deduplicating reports unless the pair is new, RSSI changes by at least 6 dB, or at least 5 seconds have elapsed.

#### Scenario: Valid ToDS client-to-AP data frame
- **WHEN** an 802.11 Data frame arrives with ToDS=1, FromDS=0, Address 1 as AP BSSID and Address 2 as Client MAC
- **THEN** an association between the client station and AP BSSID is recorded and emitted

#### Scenario: Valid FromDS AP-to-client data frame
- **WHEN** an 802.11 Data frame arrives with ToDS=0, FromDS=1, Address 1 as Client MAC and Address 2 as AP BSSID
- **THEN** an association between the client station and AP BSSID is recorded and emitted

#### Scenario: Multicast or broadcast station ignored
- **WHEN** a data frame carries a multicast or broadcast address for the client station
- **THEN** the frame is ignored and no association telemetry is emitted

#### Scenario: Association deduplication
- **WHEN** the same client and BSSID pair is observed again within 5 seconds and less than 6 dB RSSI delta
- **THEN** the duplicate observation is suppressed without emitting telemetry
