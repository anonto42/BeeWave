# web-client Specification

## Purpose

An offline-capable PWA that connects to the device over Web Bluetooth and presents status, control and scan results through independent feature slices.

## Requirements

### Requirement: Transport abstraction
Features SHALL talk to the device only through the `Transport` port so the BLE implementation and the mock are interchangeable.

#### Scenario: Mock mode
- **WHEN** the page is opened with `?mock`
- **THEN** the UI runs against a simulated device without Bluetooth

### Requirement: Isolated feature slices
Each feature under `web/src/features/` SHALL depend only on `shared/` and MUST NOT import another feature.

#### Scenario: Adding a feature
- **WHEN** a new feature module is added and mounted in `main.ts`
- **THEN** no existing feature changes

### Requirement: Offline shell
The production build SHALL be cached by a service worker so the app starts without network access.

#### Scenario: Cold offline start
- **WHEN** the installed app is opened with no connectivity
- **THEN** the shell loads from the cache

### Requirement: Firmware update over BLE
The web client SHALL send a firmware image in acknowledged chunks with a bounded in-flight window, SHALL include the image SHA-256, and MUST resend from the acknowledged offset after a lost chunk or timeout.

#### Scenario: Lost chunk
- **WHEN** a chunk is lost in transit
- **THEN** the upload resumes from the device's acknowledged offset and completes with an identical image

#### Scenario: Silent device
- **WHEN** the device never acknowledges the begin command
- **THEN** the upload fails with an error instead of hanging

### Requirement: Target tray
The client SHALL let the user mark scanned networks as targets, show them in a tray, and keep them across page reloads without failing when browser storage is unavailable.

#### Scenario: Storage blocked
- **WHEN** browser storage throws on read or write
- **THEN** targets still work in memory

### Requirement: Unsupported browser notice
The client SHALL tell the user when Web Bluetooth is unavailable and point to a supported browser.

#### Scenario: Safari on iPhone
- **WHEN** the page loads in a browser without Web Bluetooth
- **THEN** a notice names Chrome/Edge and Bluefy

### Requirement: Device key and automatic authentication
The client SHALL authenticate automatically after connecting, SHALL claim an unclaimed device by generating a random key and saving it only after the device accepts it, SHALL let the user enter a key by hand for an already claimed device, and SHALL keep the key in browser storage without failing when storage is unavailable.

#### Scenario: Key stored
- **WHEN** the user saves a valid 64 hex character key and connects to a claimed device
- **THEN** the client completes the handshake without further input

#### Scenario: First connect
- **WHEN** the device reports it is unclaimed
- **THEN** the client provisions a new key, saves it after acceptance and the session is authenticated

#### Scenario: Claimed by someone else
- **WHEN** the device is claimed and the browser has no key
- **THEN** the UI shows that a key is needed

#### Scenario: Wrong key
- **WHEN** the device rejects the proof
- **THEN** the UI shows that authentication failed

### Requirement: Signed update upload
The client SHALL send the image's signature file with the update, SHALL stop immediately with the device's reason when the device reports an OTA error, and MUST only report success after the device confirms the signed end.

#### Scenario: Refused signature
- **WHEN** the device reports a bad signature
- **THEN** the upload fails at once with that message instead of waiting for a timeout

### Requirement: PMKID display and Hashcat export
The client SHALL display captured PMKID values associated with targeted access points and SHALL offer export in standard Hashcat format (`WPA*01*...`).

#### Scenario: PMKID received for target
- **WHEN** a `PmkidFound` event matches a targeted BSSID
- **THEN** the PMKID is displayed in the handshake panel with a Hashcat export option

### Requirement: Discovered client devices view
The client SHALL display discovered client devices, showing client MAC, vendor resolution (distinguishing locally administered/randomized MACs), RSSI, channel, and their list of requested SSIDs.

#### Scenario: Probe telemetry received
- **WHEN** `ProbeResult` telemetry is received
- **THEN** the client device table updates with the client MAC, signal strength, and probed network name

### Requirement: Connected Stations Display and Association Correlation
The web client SHALL ingest `AssocResult` telemetry messages into an association store, mapping connected client stations to Access Point targets. The web client SHALL display connected station counts and station details (MAC address, vendor OUI resolution, RSSI, and last active timestamp) in the Access Point target views.

#### Scenario: Ingesting association telemetry
- **WHEN** an `AssocResult` telemetry message is received for an Access Point BSSID
- **THEN** the client station is added to or updated in the Access Point's connected station list

#### Scenario: Displaying connected stations under target
- **WHEN** an Access Point card in the Scan tab is expanded
- **THEN** all client stations associated with that Access Point are listed with resolved vendor and signal strength

### Requirement: WPS and Hardware Details Display
The web client SHALL ingest `WpsResult` telemetry messages, storing WPS security posture and device identification per Access Point. The web client SHALL display WPS status badges (e.g., WPS version, Locked, Open, PushButton) and hardware manufacturer/model details in Access Point scanner rows and target details.

#### Scenario: Ingesting WPS metadata
- **WHEN** a `WpsResult` message is received for an Access Point BSSID
- **THEN** the Access Point record is updated with its WPS version, lockout state, and hardware details

#### Scenario: Displaying WPS audit badge
- **WHEN** an Access Point has WPS enabled
- **THEN** a WPS badge indicates whether WPS is locked against PIN attacks or active

### Requirement: Security telemetry decoding and storage
The web client SHALL decode `SecurityResult` messages received over Web Bluetooth and attach security and PMF information to the corresponding `NetworkRecord` in `ScanStore`.

#### Scenario: Telemetry message dispatch
- **WHEN** a `SecurityResult` telemetry packet is decoded
- **THEN** `ScanStore.ingestSecurity` updates the target BSSID record and notifies change listeners

### Requirement: Security badges in scanner and target views
The web client SHALL display security badges (such as `OPEN`, `WEP`, `WPA2`, `WPA3`, `OWE`, and `PMF`) on Access Point rows in Scanner and Target views.

#### Scenario: Display security badge
- **WHEN** an Access Point record has associated security information
- **THEN** the Scanner row displays a badge identifying the protocol and PMF status

### Requirement: Beacon flood alert ingestion and display
The web client SHALL decode `BeaconFloodAlert` telemetry messages and publish a high-severity alert to the Alert feed and toast system.

#### Scenario: Display beacon flood alert
- **WHEN** a `BeaconFloodAlert` packet is decoded
- **THEN** an alert with source "beaconFlood" and severity "alert" is added to `AlertStore` detailing the frame rate and unique BSSID count

### Requirement: Decloaked result ingestion and display
The web client SHALL decode `DecloakedResult` telemetry messages, update the network record in `ScanStore` with `decloaked: true`, post an informational alert to `AlertStore`, and render a `DECLOAKED` badge in the scanner view.

#### Scenario: Display decloaked badge
- **WHEN** a network has `decloaked: true` in `ScanStore`
- **THEN** the scanner row renders a `DECLOAKED` badge alongside the uncloaked SSID

#### Scenario: Alert emitted on decloak
- **WHEN** a `DecloakedResult` telemetry message is received
- **THEN** an alert is recorded in `AlertStore` noting the revealed SSID and channel

### Requirement: BLE flood alert ingestion and display
The web client SHALL decode `BleFloodAlert` telemetry messages and publish a high-severity alert to the Alert feed and toast system.

#### Scenario: Display BLE flood alert
- **WHEN** a `BleFloodAlert` packet is decoded
- **THEN** an alert with source "bleFlood" and severity "alert" is added to `AlertStore` detailing the packet rate and dominant spam class

### Requirement: WiGLE.net wardriving CSV export
The web client SHALL support acquiring geolocation coordinates, associating them with scanned networks, and generating standard WiGLE v1.4 compatible CSV files for wardriving.

#### Scenario: WiGLE CSV header and formatting
- **WHEN** networks are exported in WiGLE format
- **THEN** the output includes the `WigleWifi-1.4` metadata header, standard column headers, and data rows with correctly calculated frequencies and security capability strings

#### Scenario: Network coordinates attachment
- **WHEN** the GPS tracker is active and a network is observed
- **THEN** the network record and scan log store the current latitude, longitude, and accuracy

#### Scenario: Export without active GPS
- **WHEN** networks are exported while GPS is inactive
- **THEN** coordinates default to 0.0 with 0.0 accuracy without crashing

### Requirement: Four-way handshake Hashcat Mode 22000 export
The web client SHALL support synthesizing standard Hashcat Mode 22000 `WPA*02*` format hashes from captured EAPOL 4-way handshakes and exporting them as `.22000` text files.

#### Scenario: Complete handshake Hashcat export
- **WHEN** a 4-way handshake has both Message 1 (or Message 3) and Message 2 frames captured
- **THEN** the client can export a valid Hashcat mode 22000 hash line containing MIC, AP MAC, Station MAC, ESSID hex, ANonce, zeroed-MIC EAPOL frame, and message pair flag

#### Scenario: Incomplete handshake without ANonce or M2
- **WHEN** a handshake lacks Message 2 or lacks an ANonce source (Message 1 or 3)
- **THEN** mode 22000 export is unavailable or disabled for that handshake pair

### Requirement: One-click clipboard copy for Hashcat hashes
The web client SHALL provide a 1-click copy action for both PMKID (`WPA*01*`) and 4-way handshake (`WPA*02*`) Hashcat Mode 22000 hashes, copying the formatted hash line to the clipboard with momentary visual confirmation.

#### Scenario: Copying PMKID hash
- **WHEN** the user triggers the copy action on a captured PMKID record
- **THEN** the `WPA*01*...` line is written to the system clipboard and the button shows visual confirmation

#### Scenario: Copying handshake hash
- **WHEN** the user triggers the copy action on a captured 4-way handshake
- **THEN** the `WPA*02*...` line is written to the system clipboard and the button shows visual confirmation

### Requirement: Unified target workspace and channel locking
The web client SHALL provide a 1-click action to pin the radio to a target's operating channel for reliable handshake capture, and SHALL present guidance when multiple targets operate across different channels.

#### Scenario: Pinning radio to target channel
- **WHEN** the user activates channel lock on a target operating on channel N
- **THEN** a configureRadio command is sent to the device to pin to channel N and the target indicates channel focus

#### Scenario: Multiple targets on distinct channels
- **WHEN** targets have distinct operating channels
- **THEN** the interface informs the user that the sniffer is hopping and provides per-target channel focus options

### Requirement: Crackable handshake state and early export
The web client SHALL evaluate whether captured handshake frames provide sufficient cryptographic material (M1+M2 or PMKID) for offline cracking, display clear status indicators, and enable export and copy actions immediately without requiring all 4 messages.

#### Scenario: M1 and M2 captured
- **WHEN** a handshake pair contains Message 1 (ANonce) and Message 2 (SNonce + MIC)
- **THEN** the status indicates crackable readiness and Hashcat mode 22000 export and copy actions are enabled

#### Scenario: Incomplete frames
- **WHEN** a handshake has only Message 1 without PMKID and without Message 2
- **THEN** the status indicates waiting for client response and export actions remain disabled

### Requirement: Disassociation alert ingestion and presentation
The web client SHALL decode `DisassocAlert` telemetry messages, add them to `AlertStore`, and display them in the Alert feed and toast system with the source BSSID and observed frame rate.

#### Scenario: Ingesting disassociation alert
- **WHEN** a `DisassocAlert` packet is received and decoded
- **THEN** an alert is recorded in `AlertStore` with source `disassoc` and formatted with the BSSID and rate

#### Scenario: High severity threshold
- **WHEN** the disassociation count reaches or exceeds 50 frames per second
- **THEN** the alert severity is elevated to alert level (red)

### Requirement: Apple Find My tracker visualization
The web client SHALL decode `BleStats` with `findMy` counts, display Apple Find My (AirTag) in the BLE Monitor table with rolling sparklines, and format tracker spike warnings in the alert feed.

#### Scenario: Display Find My counts
- **WHEN** `BleStats` telemetry is received
- **THEN** the BLE Monitor table displays active counts and history for Apple Find My

#### Scenario: Find My spike alert
- **WHEN** a `BleSpike` for `findMy` is received
- **THEN** an alert is posted in `AlertStore` naming Apple Find My

### Requirement: Handshake capture panel
The client SHALL show captured BSSID/client pairs with which messages were seen, SHALL let the owner turn the capture on or off, and SHALL offer a hex export of the captured EAPOL bytes with any truncated message marked.

#### Scenario: Off by default
- **WHEN** the device has not been told to enable handshake capture
- **THEN** no handshake telemetry is requested or shown

### Requirement: Passive monitoring panels
The client SHALL provide an expanded results table, channel occupancy and utilisation, RSSI history with a delta view, deauth, BLE and anomaly panels, scan comparison with JSON export, a phone-local watchlist, and a session export in JSON and CSV, all built from telemetry already received and without sending radio-affecting commands.

#### Scenario: Delta view
- **WHEN** a network's RSSI moves by at least 6 dB since the previous cycle
- **THEN** it is highlighted in the delta view

#### Scenario: Session export
- **WHEN** the user exports the session
- **THEN** a JSON and a CSV file containing scans, alerts, log and authentication events are downloaded

#### Scenario: Watchlist
- **WHEN** a pinned BSSID appears, disappears or moves by at least 6 dB
- **THEN** an alert is raised and nothing is stored on the device

### Requirement: Settings management surface
The client SHALL present device-side settings in sections with one Apply and Revert bar and per-section dirty tracking, SHALL send one `SaveSettingsAll` on Apply and show the reason and highlight the field on rejection, SHALL save phone-side settings immediately, and SHALL load device settings only after authentication.

#### Scenario: Rejected save
- **WHEN** the device answers `SettingsSaved` with `ok: false`
- **THEN** the bar shows the reason, the field is highlighted and the confirmed values are unchanged

#### Scenario: Before authentication
- **WHEN** the link is up but authentication has not succeeded
- **THEN** no settings request is sent

### Requirement: Settings import and export
The client SHALL export phone-side settings (and optionally history) as JSON with a schema version, SHALL import such a file only after showing the differences, SHALL reject unknown keys, and SHALL export device settings as a separate file.

#### Scenario: Unknown key
- **WHEN** an imported file contains an unknown key
- **THEN** the import is refused and nothing changes

### Requirement: Evil Twin security downgrade alert presentation
The web client SHALL decode `SecurityDowngrade` anomaly alerts and render them in the alert feed with alert-level severity (red), identifying the rogue BSSID and explaining that an unencrypted clone of a secured network was detected.

#### Scenario: Display Evil Twin alert
- **WHEN** an `AnomalyAlert` packet with kind `securityDowngrade` is decoded
- **THEN** an alert is posted in `AlertStore` with severity `alert` and title `Evil Twin security downgrade detected`

### Requirement: Batch PCAP and consolidated Hashcat 22000 multi-target export
The web client SHALL support combining all captured 4-way handshakes into a single chronologically-ordered libpcap (`.pcap`) file and consolidating all crackable handshakes and clientless PMKIDs into a single unified Hashcat Mode 22000 file with 1-click download and clipboard actions.

#### Scenario: Exporting all handshakes to a single PCAP
- **WHEN** multiple 4-way handshakes have been captured across different BSSIDs or clients and the user requests all PCAP export
- **THEN** the client generates and downloads a single `.pcap` file containing all captured EAPOL frames from all pairs ordered chronologically

#### Scenario: Exporting all handshakes and PMKIDs to a single 22000 file
- **WHEN** multiple crackable handshakes or PMKIDs have been captured and the user requests all 22000 export
- **THEN** the client generates and downloads a consolidated `.22000` text file containing standard `WPA*02*` and `WPA*01*` formatted lines for all valid captures

#### Scenario: Copying all 22000 hashes to clipboard
- **WHEN** crackable handshakes or PMKIDs exist and the user clicks copy all 22000
- **THEN** the consolidated 22000 formatted lines are copied to the system clipboard with momentary visual confirmation feedback

#### Scenario: No crackable handshakes or PMKIDs
- **WHEN** no crackable handshakes or PMKIDs have been captured
- **THEN** batch export controls remain hidden or disabled in the workspace toolbar

### Requirement: Probe flood alert ingestion and presentation
The web client SHALL decode `ProbeFloodAlert` telemetry messages and display them in the Alert feed and toast system with alert-level severity (red), indicating the observed probe frame rate and unique client MAC count.

#### Scenario: Ingesting probe flood alert
- **WHEN** the web client receives a `probeFloodAlert` telemetry message
- **THEN** it formats an alert with title indicating rapid probe request flood, detail displaying the rate and client count, and severity `alert`

### Requirement: KARMA SSID change alert presentation
The web client SHALL decode `KarmaSsidChange` anomaly alerts and render them in the alert feed with alert-level severity (red), identifying the BSSID and explaining that an active KARMA/MANA rogue AP or SSID honeypot mutation was detected.

#### Scenario: Ingesting KARMA anomaly alert
- **WHEN** the web client receives an `anomaly` telemetry message with kind `karmaSsidChange`
- **THEN** it formats an alert with title "KARMA / Rogue AP SSID change detected" and severity `alert`
