# Design

## Context
See `proposal.md` for motivation. Currently, `firmware/src/adapters/wifi_radio/callback.rs` filters 802.11 management frames and checks only `frame[0] == DEAUTH_FC0` (`0xC0`), recording counts into a lock-free table (`wifi_radio/deauth.rs`). Once per second, the application task drains these counts and feeds them to `App::on_deauth_window`, which evaluates thresholds in `Monitoring` and publishes `DeauthAlert`.

802.11 Disassociation frames (subtype `0xA0`) share the identical MAC header structure with Deauthentication frames: Frame Control (2 bytes), Duration (2 bytes), Address 1 (RA/Destination, 6 bytes), Address 2 (TA/Source, 6 bytes), Address 3 (BSSID, 6 bytes at offset 16), and Sequence Control (2 bytes).

## Goals / Non-Goals

**Goals:**
- Detect 802.11 Disassociation frame floods (subtype `0xA0`) in real time.
- Provide a dedicated `DisassocAlert` telemetry variant so operators know whether attacks use deauthentication or disassociation frames.
- Maintain zero-allocation and lock-free execution in the Wi-Fi promiscuous callback.
- Zero extra heap on the ESP32; static RAM cost under 100 bytes.

**Non-Goals:**
- Transmitting disassociation or deauth frames (BeeWave is strictly receive-only).
- Client-side packet injection or 802.11 frame modification.

## Decisions

### Decision 1: Dedicated `DisassocAlert` variant vs overloaded `DeauthAlert`
- *Choice*: Add a dedicated `DisassocAlert { bssid: [u8; 6], count: u16 }` variant to `TelemetryMessage`.
- *Rationale*: Attackers often test target behavior by varying between deauth and disassociation. Differentiating between them allows network auditors to identify the precise tool profile and vector being deployed against the network.
- *Alternative*: Merging into `DeauthAlert` was considered, but obscures which frame subtype is actually being flooded.

### Decision 2: Driver-level lock-free counting table
- *Choice*: Mirror the lock-free `Slot` table pattern from `wifi_radio/deauth.rs` for disassociation frames or generalize it to support subtype tagging without extra memory overhead.
- *Rationale*: The promiscuous callback runs in the Wi-Fi driver task context and cannot acquire mutexes or block. The atomic slot table guarantees safe multi-task counts with negligible CPU overhead.
- *RAM cost*: 8 slots $\times$ 10 bytes = 80 bytes static BSS memory.

### Decision 3: Alert severity and presentation
- *Choice*: In `web/src/shared/monitor/alerts.ts`, disassociation floods $>=$ configured threshold are flagged as warning (`warn`), and elevated to `alert` (red) if exceeding 50 frames/second (identical to deauth flood rules).

## Risks / Trade-offs

- [Risk: High frame rate during severe flood overwhelms BLE link] $\to$ Bounded table drains at 1 Hz and limits alert emissions to at most 4 BSSIDs per window, bounded by `heapless::Vec<_, 4>`, protecting BLE throughput.
- [Risk: Flash budget] $\to$ Minimal code delta; uses existing parser utilities. Flash impact < 500 bytes.
