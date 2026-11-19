# Proposal

## Why

Currently, information required to audit Wi-Fi networks is scattered across disconnected tabs (`Scan`, `Analyze`, `Alerts`), while target selection and handshake capture lack a clear, intuitive operational flow. Users face three major pain points:
1. **Full 4-Way vs Crackable Handshake Confusion**: In real-world Wi-Fi, Message 1 + Message 2 contains the entire cryptographic challenge (ANonce, SNonce, MIC) required for Hashcat Mode 22000 offline cracking. The previous UI marked M1+M2 as "partial" and kept export buttons disabled unless all 4 messages were observed, leading users to believe the capture failed.
2. **Channel Hopping vs Channel Lock**: If the radio hops across 13 channels during handshake capture, it misses rapid 4-way exchanges 92% of the time. Users need clear visibility into whether the sniffer is hopping or locked, with a 1-click action to pin the radio to the target's channel.
3. **Fragmented Target Intelligence**: Checking connected client stations, client probe requests (PNL), deauth alerts, and handshake progress currently requires switching between 3 different tabs.

A unified Target Workspace brings all relevant target intelligence together, provides transparent single vs multi-target channel locking, and clearly illustrates the step-by-step handshake capture progression.

## What Changes

- **Crackable State & Instant Export**: Update handshake tracking to recognize `M1+M2` as `CRACKABLE` for Hashcat mode 22000 and enable `.22000`, `COPY`, and `.PCAP` exports as soon as M1+M2 or PMKID is captured.
- **One-Click Target Channel Lock**: Add a `[LOCK CH <N>]` / `[AUTO HOP]` button directly in the target card to immediately pin the ESP32 radio (`configureRadio`) to the target's frequency for 100% reliable capture, with clear multi-target channel hopping guidance.
- **Unified Target Context**: Integrate active client stations (with vendor OUI), client probe requests (PNL), and target-specific deauth alerts directly into the target card.
- **Step-by-Step Handshake Progress**: Display transparent message badges (`M1: ANonce`, `M2: SNonce & MIC`, `M3: Keys`, `M4: Done`) with live status (`WAITING FOR CLIENT`, `CRACKABLE (M1+M2)`, `FULL (4/4)`, `PMKID READY`).

## Capabilities

### Modified Capabilities

- `web-client`: Add requirements for unified target workspace, channel locking from target cards, crackable M1+M2 handshake status recognition, and integrated target telemetry.

## Impact

- Files affected:
  - `web/src/features/handshake/template.ts`
  - `web/src/features/handshake/mount.ts`
  - `web/src/features/scanner/template.ts`
  - `web/src/features/scanner/mount.ts`
  - `web/src/style.css`
- Core firmware (`crates/core`) and wire protocol (`crates/protocol`) are already fully capable and remain unchanged (`configureRadio`, `setHandshakeCapture`, `HandshakeFrame`, `HandshakeComplete`).
