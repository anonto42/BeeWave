# device-core Specification

## Purpose

Hardware-independent device behaviour (radio control, scan reporting, status, OTA) implemented in `crates/core` behind ports.

## Requirements

### Requirement: Validated radio control
The radio feature SHALL accept channels 1 through 13, or 0 meaning automatic hopping, and MUST reject any other channel without calling the driver. Status SHALL report the channel the radio is actually on.

#### Scenario: Invalid channel
- **WHEN** `ConfigureRadio` carries channel 99
- **THEN** the command fails with an invalid-channel error and the current channel is unchanged

### Requirement: Beacon parsing
The scan feature SHALL parse 802.11 beacon and probe-response frames into a BSSID, SSID, channel and RSSI without allocating, and MUST ignore other frame types and malformed or truncated frames without panicking.

#### Scenario: Hidden network
- **WHEN** a beacon carries an empty or all-zero SSID
- **THEN** the observation has an empty SSID

#### Scenario: Truncated element
- **WHEN** an information element claims more bytes than the frame holds
- **THEN** parsing stops and returns what was read so far

### Requirement: De-duplicated scan reporting
The scan feature SHALL report a network only when it is new or its RSSI moved by at least 6 dB, and SHALL bound its table without allocating.

#### Scenario: Stable network
- **WHEN** the same BSSID is observed again within 6 dB
- **THEN** no telemetry is emitted

#### Scenario: Table full
- **WHEN** more distinct networks are seen than the table can hold
- **THEN** the table evicts and continues reporting without failing

### Requirement: Sequential acknowledged OTA
The OTA feature SHALL accept chunks only at the expected offset, SHALL bound total size to the announced size, and MUST NOT finish unless all bytes were received.

#### Scenario: Out-of-order chunk
- **WHEN** a chunk arrives at the wrong offset
- **THEN** it is not written and an acknowledgement carries the expected offset

#### Scenario: Chunk without begin
- **WHEN** a chunk arrives with no active session
- **THEN** it is rejected as a bad-state error

### Requirement: Lossy telemetry
The core SHALL never block on the telemetry sink and SHALL count dropped messages and report the count in status.

#### Scenario: Sink full
- **WHEN** the sink rejects messages
- **THEN** the dropped counter increases and the app continues

### Requirement: Persistent settings
The settings feature SHALL validate the channel (0 for auto, or 1 through 13), SHALL save settings through the storage port before accepting them, and MUST keep the previous settings when storage fails. Saved settings SHALL be applied at startup.

#### Scenario: Invalid channel
- **WHEN** `SetSettings` carries channel 99
- **THEN** it is rejected, nothing is saved and an error event is emitted

#### Scenario: Storage failure
- **WHEN** the storage port fails to save
- **THEN** the in-memory settings are unchanged and an error event is emitted

#### Scenario: Boot
- **WHEN** the device starts with saved settings
- **THEN** the saved channel is applied and scanning starts if scan-at-boot was saved

### Requirement: Authentication gate
The core SHALL verify proofs in constant time, SHALL use each nonce once, and MUST drop commands and non-auth telemetry while unauthenticated.

#### Scenario: Wrong proof
- **WHEN** the proof does not match
- **THEN** the result is not ok and the session stays unauthenticated

#### Scenario: Locked gate
- **WHEN** the gate has no key
- **THEN** every proof fails

### Requirement: Signed firmware updates
The OTA feature SHALL finish an update only after an Ed25519 signature over the announced image SHA-256 verifies against the built-in public key, MUST abort the update on a missing or invalid signature, and MUST refuse updates entirely when no public key is configured.

#### Scenario: Bad signature
- **WHEN** the signature does not verify
- **THEN** the update is aborted, the image is never finalised, and the client gets a bad-signature error

#### Scenario: Unsigned end
- **WHEN** the client ends an update without a signature
- **THEN** the update is aborted and refused as unsigned

#### Scenario: No signing key
- **WHEN** the device has no OTA public key
- **THEN** beginning an update is refused

### Requirement: Latency statistics
The core SHALL aggregate latency samples into a fixed-size histogram without allocating and SHALL report upper bounds for the 50th and 99th percentiles and the maximum, returning no percentile when there are no samples.

#### Scenario: Slow tail
- **WHEN** two percent of samples are far slower than the rest
- **THEN** the 99th percentile reports the slow bucket

#### Scenario: Above the last bucket
- **WHEN** a sample exceeds the last bucket edge
- **THEN** the percentile reports the true maximum

### Requirement: Configurable hop order
The radio feature SHALL select the next hop channel from a supplied order, wrapping at the end, and SHALL restart from the first entry when the current channel is not in the order.

#### Scenario: Wrap
- **WHEN** the current channel is the last in the order
- **THEN** the next channel is the first in the order

### Requirement: Passive PMKID extraction
The core SHALL inspect the Key Data field of parsed EAPOL-Key Message 1 frames for PMKID Key Data Encapsulation (KDE) or RSN Information Elements without transmitting any frames, and SHALL emit `PmkidFound` when a valid PMKID is detected.

#### Scenario: EAPOL-Key M1 with PMKID KDE
- **WHEN** an EAPOL-Key Message 1 contains a PMKID KDE (`00-0F-AC-04`)
- **THEN** the core extracts the 16-byte PMKID, BSSID, and client MAC address

#### Scenario: Frame without PMKID
- **WHEN** an EAPOL frame contains no PMKID KDE
- **THEN** no PMKID telemetry is emitted

### Requirement: PMKID deduplication
The core SHALL maintain a bounded cache of recently seen PMKIDs and SHALL suppress redundant telemetry emissions for previously observed PMKID entries.

#### Scenario: Duplicate PMKID
- **WHEN** an identical PMKID is seen again for the same BSSID
- **THEN** the duplicate frame is ignored and not republished

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

### Requirement: Zero-allocation Wi-Fi security suite and PMF parsing
The core scan engine SHALL parse 802.11 Capability Information flags, RSN elements (Tag 48), and WPA elements (Tag 221) from Beacon and Probe Response frames without heap allocation to extract security protocol, AKM, cipher suite, and Protected Management Frame (PMF) mode.

#### Scenario: Open network without privacy bit
- **WHEN** a beacon is received with Privacy bit cleared (0) and no RSN/WPA elements
- **THEN** the core identifies the security protocol as Open and PMF mode as None

#### Scenario: WPA2-PSK network with CCMP
- **WHEN** a beacon is received with an RSN element specifying AKM suite 00-0F-AC:2 (PSK) and pairwise cipher CCMP
- **THEN** the core identifies the security protocol as WPA2-Personal with CCMP cipher

#### Scenario: WPA3-SAE network with mandatory PMF
- **WHEN** a beacon is received with an RSN element specifying AKM suite 00-0F-AC:8 (SAE) and RSN capabilities with MFPR=1 and MFPC=1
- **THEN** the core identifies the security protocol as WPA3-Personal and PMF mode as Required

#### Scenario: WPA2/WPA3 transition mode
- **WHEN** a beacon is received with an RSN element specifying both PSK (00-0F-AC:2) and SAE (00-0F-AC:8) AKMs with MFPC=1 and MFPR=0
- **THEN** the core identifies the security protocol as WPA2/WPA3 Transition and PMF mode as Capable

### Requirement: Security observation deduplication
The core SHALL track observed security postures in a bounded deduplication table (`SecurityTracker`) and emit `SecurityResult` messages only when an Access Point is first observed or its security attributes (protocol, PMF, AKM, cipher) change.

#### Scenario: Duplicate beacons with identical security
- **WHEN** repeated beacons from the same BSSID are received with unchanged security and PMF attributes
- **THEN** the core does not re-emit `SecurityResult` messages over the BLE sink

#### Scenario: Security configuration change or downgrade
- **WHEN** a beacon for a known BSSID is received with altered security protocol or PMF attributes
- **THEN** the core immediately emits an updated `SecurityResult` message

### Requirement: Passive beacon flood detection
The core monitoring engine SHALL track beacon frame arrival velocity and unique BSSID counts per 1-second interval without heap allocation and emit a `BeaconFloodAlert` when the rate exceeds the flood threshold.

#### Scenario: Normal beacon reception below threshold
- **WHEN** beacon frames arrive at normal rates (below 25 frames per second)
- **THEN** no `BeaconFloodAlert` is emitted

#### Scenario: Beacon flood attack exceeds threshold
- **WHEN** beacon frames arrive exceeding 25 frames per second with at least 10 unique BSSIDs in a 1-second window
- **THEN** the core emits a `BeaconFloodAlert` carrying the rate and unique BSSID count

#### Scenario: Flood alert cooldown rate limiting
- **WHEN** a beacon flood continues continuously over multiple consecutive seconds
- **THEN** the core rate-limits `BeaconFloodAlert` emissions to at most one alert every 5 seconds to prevent BLE saturation
