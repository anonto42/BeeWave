# Proposal

## Why

In WPA2/WPA3 networks supporting PMK caching (IEEE 802.11i), an Access Point transmitting EAPOL-Key Message 1 includes the PMKID in its Key Data field. Extracting the PMKID allows security researchers to verify and audit WPA-PSK passwords without needing to wait for or capture a complete 4-way handshake (messages 1 through 4) or require a client reconnection. Like the rest of BeeWave, this is strictly receive-only: the sniffer passively parses PMKID elements from naturally occurring EAPOL frames without transmitting anything.

## What Changes

- **Core parsing (`crates/core/src/features/handshake`)**:
  - Add zero-allocation, bounded parser `parse_pmkid` to inspect the Key Data portion of EAPOL-Key frames (specifically Message 1 with KDE `00-0F-AC-04` or RSN IE Tag 48).
  - Extract `[u8; 16]` PMKID alongside BSSID and client MAC.
- **Wire protocol (`crates/protocol`)**:
  - Append a new telemetry variant `PmkidFound { bssid: [u8; 6], client: [u8; 6], pmkid: [u8; 16] }` to `TelemetryMessage`. Protocol version remains 1 (append-only).
- **Core tracking (`crates/core/src/features/handshake`)**:
  - Track seen PMKIDs in a bounded `heapless` table to deduplicate and prevent telemetry flooding.
- **Web client (`web/src`)**:
  - Mirror protocol decode/encode in `web/src/shared/protocol/`.
  - In `web/src/features/handshake`, display captured PMKIDs under the targeted network.
  - Add export in modern Hashcat format (`*01*` line format: `WPA*01*<pmkid>*<bssid>*<client>*<essid>***`).

## Non-Goals & Invariants

- **Strictly Receive-Only**: BeeWave does not initiate associations or send probe requests to trigger EAPOL frames from APs (unlike active attack tools). It only captures PMKIDs transmitted during legitimate client connections.
- **Zero Allocation**: Parsing is performed directly on byte slices (`&[u8]`) using fixed-size arrays without heap allocation.
- **Memory Bounded**: PMKID deduplication table uses fixed capacity (`heapless`).

## Capabilities

### Modified Capabilities
- `device-core`: Passive extraction and deduplication of PMKID from EAPOL frames.
- `wire-protocol`: Telemetry variant `PmkidFound` appended to `TelemetryMessage`.
- `web-client`: UI display of captured PMKIDs and Hashcat 22000 export.

## Impact

- `crates/protocol`: Appends `PmkidFound` to `TelemetryMessage`; golden vectors updated.
- `crates/core`: `features/handshake/` gains PMKID KDE parser and deduplication.
- `web`: `shared/protocol/` mirrors wire codec; `features/handshake` renders PMKID and export action.
- Docs: Update `docs/MONITORING.md`.
