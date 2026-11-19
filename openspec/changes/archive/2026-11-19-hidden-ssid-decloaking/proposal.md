# Proposal

## Why
In 802.11 wireless networks, access points configured with "hidden SSID" broadcast beacon frames containing an empty (0-length) or null-padded SSID tag. Wireless penetration testers and security auditors identify these networks by passively sniffing directed probe requests from authorized client stations or directed probe responses from the AP itself, which reveal the network's true SSID in cleartext.

Currently, BeeWave's `ScanTable` in `crates/core` records hidden access points with empty SSIDs and filters subsequent observations based strictly on RSSI movement (`>= 6 dB`) and minimum reporting intervals (`scan_interval_ms`). Consequently, when a directed probe response or beacon reveals the uncloaked SSID, `ScanTable` drops the frame if RSSI has not moved sufficiently. Furthermore, the web client lacks explicit telemetry, notification events, and UI badges indicating that a cloaked AP has been discovered and uncloaked.

## What Changes
- Add `DecloakedResult { bssid: [u8; 6], ssid: String<32>, channel: u8, rssi: i8 }` to `TelemetryMessage` in `crates/protocol`.
- In `crates/core/src/features/scan/table.rs`, track whether an AP entry was heard with an empty SSID. When an observation provides a non-empty SSID for an entry previously recorded as empty:
  - Force an immediate report, bypassing RSSI deduplication and interval restrictions.
  - Emit `TelemetryMessage::DecloakedResult` carrying the newly revealed SSID, channel, and RSSI.
  - Update the entry state so subsequent empty beacons cannot revert the known SSID to empty.
- Update golden wire test vectors (`make vectors`) and mirror the decoder in `web/src/shared/protocol/`.
- In `web/src/shared/monitor/scan-store.ts`, update `NetworkRecord` to support an `uncloaked: boolean` flag, and update `ingest` / `ingestDecloaked` to flag uncloaked networks.
- In `web/src/shared/monitor/alerts.ts` and `AlertStore`, format and display informational alerts when hidden networks are uncloaked (`[DECLOAKED] Hidden network <BSSID> revealed as "<SSID>" on CH <channel>`).
- In `web/src/features/scanner/template.ts`, render a distinct `DECLOAKED` badge for uncloaked networks.

## Capabilities

### Modified Capabilities
- `device-core`: Detect uncloaking of hidden SSIDs from probe responses or beacons, bypassing RSSI deduplication, and emitting `DecloakedResult`.
- `wire-protocol`: Add `DecloakedResult` variant to `TelemetryMessage` and update golden test vectors.
- `web-client`: Ingest `DecloakedResult` telemetry, update `ScanStore` records with `uncloaked: true`, post informational decloaking alerts, and render `DECLOAKED` badges in the scanner list.

## Impact
- `crates/protocol`: Appends `DecloakedResult` variant to `TelemetryMessage` (non-breaking append).
- `crates/core`: Modifies `ScanTable` internal entry storage to track empty SSID state, returning `DecloakedResult` when decloaked.
- `web`: Decodes `decloakedResult` in protocol parser, updates scanner rendering, and publishes decloak events in `AlertStore`.
