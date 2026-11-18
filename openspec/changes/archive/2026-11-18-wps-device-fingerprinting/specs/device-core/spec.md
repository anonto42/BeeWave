# Spec Delta

## ADDED Requirements

### Requirement: Passive WPS Information Element Parsing
The scan feature SHALL inspect vendor-specific information elements (element ID 221 with OUI `00:50:f2:04`) in 802.11 Beacon and Probe Response frames without allocating memory. It SHALL extract WPS version, setup state, lockout flag, selected registrar flag, device name, manufacturer, and model name, emitting deduplicated observations per Access Point BSSID.

#### Scenario: Valid WPS vendor element
- **WHEN** a beacon carries a vendor element with OUI 00:50:f2:04 containing device and setup attributes
- **THEN** WPS metadata is extracted and a `WpsResult` is emitted for the BSSID

#### Scenario: Locked WPS status
- **WHEN** an AP advertises the AP Setup Locked attribute (0x1057) with value 1
- **THEN** the observation flags the Access Point as locked

#### Scenario: Malformed or truncated WPS element
- **WHEN** a vendor element claims more bytes than the frame contains or has invalid TLV lengths
- **THEN** parsing terminates safely without panicking, extracting only valid attributes read so far

#### Scenario: Deduplication of WPS metadata
- **WHEN** consecutive beacons from the same BSSID carry identical WPS attributes
- **THEN** duplicate telemetry emission is suppressed
