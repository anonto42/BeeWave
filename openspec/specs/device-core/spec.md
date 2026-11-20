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
The core SHALL verify proofs in constant time, SHALL use each nonce once, SHALL accept a key from a client only while it has none (persisting it before claiming), and MUST drop commands and non-auth telemetry while unauthenticated.

#### Scenario: Wrong proof
- **WHEN** the proof does not match
- **THEN** the result is not ok and the session stays unauthenticated

#### Scenario: Claim once
- **WHEN** an unclaimed device receives a valid `Provision`
- **THEN** the key is stored, the session is authenticated and a second `Provision` is refused

#### Scenario: Storage failure
- **WHEN** the key cannot be stored
- **THEN** the device stays unclaimed and reports failure

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

### Requirement: Hidden SSID automatic decloaking
The core scan table SHALL track whether a known BSSID was originally heard with an empty SSID, and when a subsequent frame provides a non-empty SSID for that BSSID, SHALL bypass deduplication thresholds and emit `TelemetryMessage::DecloakedResult`.

#### Scenario: Network heard initially with empty SSID
- **WHEN** an observation for a new BSSID has an empty SSID
- **THEN** it is inserted into `ScanTable` with `had_empty_ssid = true` and emitted as `ScanResult`

#### Scenario: Hidden network uncloaked by probe response
- **WHEN** a subsequent observation for that BSSID has a non-empty SSID, even if RSSI delta is under threshold or minimum interval has not elapsed
- **THEN** `ScanTable` updates the entry, marks `had_empty_ssid = false`, and returns `TelemetryMessage::DecloakedResult`

#### Scenario: Empty beacon does not overwrite known SSID
- **WHEN** an entry already has a known non-empty SSID and a subsequent observation arrives with an empty SSID
- **THEN** `ScanTable` does not revert `had_empty_ssid` to true and retains the known SSID status

### Requirement: Real-time BLE flood detection
The core monitoring engine SHALL track BLE advertising arrival rates per 1-second interval without heap allocation and emit a `BleFloodAlert` carrying the rate and dominant advertisement class when the rate exceeds the flood threshold.

#### Scenario: Normal BLE advertisement rate below threshold
- **WHEN** BLE advertisement frames arrive at normal ambient rates (below 30 packets per second)
- **THEN** no `BleFloodAlert` is emitted

#### Scenario: BLE spam attack exceeds threshold
- **WHEN** BLE advertisement frames arrive exceeding 30 packets per second in a 1-second window
- **THEN** the core emits a `BleFloodAlert` carrying the rate and dominant advertisement class

#### Scenario: Flood alert cooldown rate limiting
- **WHEN** a BLE flood attack continues continuously over multiple consecutive seconds
- **THEN** the core rate-limits `BleFloodAlert` emissions to at most one alert every 5 seconds to prevent BLE saturation

### Requirement: Disassociation frame attack detection
The core SHALL track received 802.11 Disassociation management frames (subtype `0xA0`) per BSSID over a 1-second sliding window without heap allocation and SHALL raise `DisassocAlert` for each BSSID whose rate meets or exceeds the configured threshold.

#### Scenario: Disassociation threshold crossed
- **WHEN** disassociation frames for a BSSID reach or exceed the threshold within the 1-second window
- **THEN** a `DisassocAlert` carrying the BSSID and frame count is emitted over the telemetry sink

#### Scenario: Disassociation below threshold or disabled
- **WHEN** the disassociation count for a BSSID is below the threshold, or the threshold is 0
- **THEN** no `DisassocAlert` is emitted

#### Scenario: Multiple BSSIDs targeted
- **WHEN** multiple BSSIDs exceed the threshold within the same window
- **THEN** an alert is emitted for each qualifying BSSID up to the sink capacity

### Requirement: Apple Find My and AirTag BLE classification
The core SHALL inspect Bluetooth Low Energy advertisement manufacturer data for Apple Company ID (`0x004C`) with Continuity sub-type `0x12` (`APPLE_FIND_MY`), classify matching packets as `AdClass::FindMy`, track their rolling counts across 1-minute windows, and evaluate spike conditions.

#### Scenario: AirTag advertisement received
- **WHEN** a BLE advertisement with Apple manufacturer data type 0x12 is observed
- **THEN** it is classified as `FindMy` and credited to the Find My class counter

#### Scenario: Find My spike alert
- **WHEN** Find My advertisement counts exceed the baseline by the configured spike factor after warm-up
- **THEN** a `BleSpike` naming `AdClass::FindMy` is published

### Requirement: Receive-only handshake classification
The core SHALL classify a parsed EAPOL-Key frame as message 1, 2, 3 or 4 of the four-way handshake from its key-info bits, MUST NOT transmit anything, and SHALL discard frames that are not EAPOL-Key frames of a recognized message.

#### Scenario: Each message number
- **WHEN** key-info bits match message 1, 2, 3 or 4
- **THEN** the frame is classified accordingly

#### Scenario: Unrecognized key-info
- **WHEN** the bits match none of the four messages (for example a group-key frame)
- **THEN** the frame is not reported

### Requirement: Per-pair capture tracking
The core SHALL track which handshake messages have been seen for each BSSID/client pair in a bounded table and SHALL report completion once a message carrying the ANonce and the message carrying the SNonce and MIC have both been seen.

#### Scenario: Completion
- **WHEN** message 2 and either message 1 or message 3 have both been seen for a pair
- **THEN** `HandshakeComplete` is reported once for that pair

### Requirement: Receive-only deauthentication detector
The core SHALL count deauthentication frames per BSSID over a one second window and SHALL raise a `DeauthAlert` for each BSSID at or above the configured threshold, and MUST NOT transmit anything.

#### Scenario: Threshold crossed
- **WHEN** a BSSID reaches the threshold within one second
- **THEN** a `DeauthAlert` with that BSSID and count is published

#### Scenario: Below threshold or disabled
- **WHEN** the count is below the threshold, or the threshold is 0
- **THEN** no alert is published

### Requirement: Receive-only BLE advertisement monitor
The core SHALL classify received advertisements as Fast Pair, iBeacon, proximity pairing or generic, SHALL keep per-class counts for the last minute, and SHALL raise `BleSpike` when a class exceeds its baseline by the configured factor after a warm-up.

#### Scenario: Classification
- **WHEN** an advertisement carries Apple manufacturer data of type 0x02 and length 0x15
- **THEN** it is classified as iBeacon

#### Scenario: Spike
- **WHEN** a class count exceeds its baseline times the factor after warm-up
- **THEN** a `BleSpike` naming the class is published

### Requirement: Anomaly alerts
The core SHALL raise `AnomalyAlert` for a known SSID on a new BSSID, a BSSID seen on two channels within a minute, an RSSI jump above 20 dB between cycles, and a BSSID returning after more than 5 missed cycles, with a cooldown per BSSID and kind.

#### Scenario: Cooldown
- **WHEN** the same anomaly repeats within the cooldown
- **THEN** it is not published again

### Requirement: Monitor configuration
The core SHALL validate `SetMonitor`, answer with `MonitorConfig`, and reject out-of-range values with an event.

#### Scenario: Invalid factor
- **WHEN** `ble_spike_factor` is outside 2..=20
- **THEN** the configuration is unchanged and an error event is published

### Requirement: Validated persistent device settings
The core SHALL validate every device-side setting against its range and the cross-field rules, SHALL persist a complete valid set and bump a revision counter, and MUST leave storage and live values untouched when a request is rejected.

#### Scenario: Out of range
- **WHEN** a field is outside its range
- **THEN** `SettingsSaved` reports `ok: false` with the reason naming the field and nothing is written

#### Scenario: Window too short
- **WHEN** `deauth_window_ms` is less than twice `scan_interval_ms`
- **THEN** the request is rejected with `WindowTooShort`

#### Scenario: Revision
- **WHEN** a save succeeds
- **THEN** the revision increases by one and survives a reboot

### Requirement: Settings take effect where they belong
The core SHALL apply scan filters and the dedup threshold to following beacons, SHALL hand hop dwell and order to the radio only when a scan starts, and SHALL use the saved status period and deauth window.

#### Scenario: Hop change mid-scan
- **WHEN** hop settings are saved while a scan is running
- **THEN** the radio keeps its current hopping until the next scan start

### Requirement: Factory reset
The core SHALL accept `FactoryReset` only from an authenticated session with no update in flight, erase settings, key and revision, and reboot.

#### Scenario: Update in flight
- **WHEN** `FactoryReset` arrives during an update
- **THEN** it is rejected with `Busy` and nothing is erased

### Requirement: Self-test
The core SHALL run a self-test of radio, NVS, BLE and OTA slot when not scanning and report each result, and MUST return `busy` without touching the radio while scanning.

#### Scenario: Scanning
- **WHEN** `RunSelfTest` arrives during a scan
- **THEN** the result has `busy: true` and no radio call is made

### Requirement: Evil Twin security downgrade anomaly detection
The core SHALL correlate the security protocol of observed access points against the known security baseline of matching SSIDs and SHALL raise `AnomalyAlert` with kind `SecurityDowngrade` when an SSID known to use robust encryption (WPA2, WPA3, Enterprise) is observed with Open or WEP security.

#### Scenario: Open rogue AP clones WPA2 network
- **WHEN** an access point is observed broadcasting an SSID previously heard with WPA2 or WPA3 encryption, but advertising Open or WEP security
- **THEN** the core emits `AnomalyAlert` with kind `SecurityDowngrade` naming the rogue BSSID

#### Scenario: Legitimate access points with matching security
- **WHEN** a new access point is observed for a known SSID with identical or compatible security posture
- **THEN** no `SecurityDowngrade` anomaly is emitted

### Requirement: Passive probe request flood detection
The core monitoring engine SHALL track received 802.11 Probe Request frame velocity and distinct client MAC counts in a 1-second sliding window without heap allocation, emitting `ProbeFloodAlert` when the rate and client diversity exceed configured thresholds.

#### Scenario: Normal probe arrival below threshold
- **WHEN** client probe requests arrive at normal ambient rates (below 30 frames per second)
- **THEN** no `ProbeFloodAlert` is emitted

#### Scenario: Probe request flood attack exceeds threshold
- **WHEN** probe requests arrive at 30 or more frames per second with at least 8 unique client MAC addresses in a 1-second window
- **THEN** the core emits `ProbeFloodAlert` containing the observed rate and unique client count

#### Scenario: Flood alert cooldown rate limiting
- **WHEN** a probe flood attack persists across consecutive 1-second intervals
- **THEN** the core rate-limits `ProbeFloodAlert` emissions to at most one alert every 5 seconds to prevent BLE link exhaustion

### Requirement: KARMA rogue AP SSID change anomaly detection
The core anomaly detector SHALL detect when an Access Point BSSID previously seen advertising a valid non-empty SSID begins broadcasting a different non-empty SSID, emitting an `AnomalyAlert` of kind `KarmaSsidChange`.

#### Scenario: Rogue AP switches advertised SSID on same BSSID
- **WHEN** an AP with a known non-empty SSID is observed broadcasting a different non-empty SSID
- **THEN** the core emits `AnomalyAlert` of kind `KarmaSsidChange` naming the offending BSSID

#### Scenario: Hidden network transition ignored
- **WHEN** an AP broadcasts an empty or hidden SSID
- **THEN** no `KarmaSsidChange` anomaly is raised

### Requirement: 802.11 PHY Standard Extraction and Deduplication
The scan feature SHALL parse 802.11 beacon and probe response frames to detect HT Capabilities (Element 45), ERP Information (Element 42), HE Capabilities (Element 255 Ext 35), and Supported Rates (Elements 1 and 50) without allocating, classify the physical standard into `WifiPhyStandard`, and emit `PhyResult` deduplicated in a bounded table (`PhyTracker`).

#### Scenario: 802.11n Wi-Fi 4 detection
- **WHEN** a beacon contains Information Element 45 (HT Capabilities)
- **THEN** the detected PHY standard is classified as `WifiPhyStandard::N` and emitted via `PhyResult`

#### Scenario: Deduplicated PHY reporting
- **WHEN** an Access Point is observed again with an unchanged PHY standard
- **THEN** `PhyTracker` suppresses duplicate telemetry emission

### Requirement: Beacon TSF Extraction and Rollback Detection
The scan feature SHALL extract the 64-bit microsecond TSF timestamp from beacon management frames, and the anomaly detector SHALL flag an `AnomalyKind::TsfRollback` alert when an Access Point's timestamp rolls backward by 5 seconds or more relative to its previous observation.

#### Scenario: Rogue AP timestamp rollback
- **WHEN** an Access Point's beacon is received with a TSF timestamp that is at least 5,000,000 microseconds less than the previously observed timestamp for that BSSID
- **THEN** the anomaly detector raises `AnomalyKind::TsfRollback` with detail set to the rollback duration in seconds

#### Scenario: Normal monotonic clock progression
- **WHEN** an Access Point's beacon is received with an incremented or equal TSF timestamp
- **THEN** no `TsfRollback` anomaly is raised
