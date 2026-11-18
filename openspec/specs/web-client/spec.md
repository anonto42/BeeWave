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
The client SHALL let the user enter the device key, keep it in browser storage without failing when storage is unavailable, and authenticate automatically after connecting.

#### Scenario: Key stored
- **WHEN** the user saves a valid 64 hex character key and connects
- **THEN** the client completes the handshake without further input

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
