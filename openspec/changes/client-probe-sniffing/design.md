# Design

## Context

802.11 Probe Request frames (`subtype 0x40`) are transmitted by client devices searching for access points.
Unlike beacon frames, Probe Requests have no fixed parameter block: Tagged Parameters (Information Elements) begin immediately at byte offset 24 after the 802.11 management MAC header.
Address 2 (bytes 10..16) contains the Transmitter Address (the client's MAC address). Element ID 0 contains the requested SSID, which has a length of 0 for wildcard/broadcast probes.

## Goals / Non-Goals

**Goals:**
- Zero-allocation, pure parsing of 802.11 Probe Request frames directly on byte slices (`&[u8]`).
- Bounded memory deduplication in `crates/core/src/features/probe/` using `heapless::Vec`.
- Distinction between directed probes (seeking specific SSIDs) and broadcast/wildcard probes.
- Presentation in the Web Bluetooth PWA showing client devices, MAC randomization status, and their Preferred Network Lists (probed SSIDs).

**Non-Goals:**
- No transmission of Probe Response frames (strictly receive-only).
- No tracking of client associations or traffic data.

## Decisions

### 1. Probe Request Frame Parsing
In 802.11 management frames:
- `frame[0] & 0xFC == 0x40` (Probe Request).
- Header length is 24 bytes (Address 1: DA, Address 2: SA/Client, Address 3: BSSID, Sequence Control).
- Element 0 (SSID):
  - Length 0: Broadcast probe (represented as empty string `""`).
  - Length 1..=32: Directed probe for that SSID.
- Element 3 (DS Parameter Set): Channel number if present; otherwise defaults to `rx_channel`.

### 2. Bounded Client Deduplication (`ProbeTracker`)
To protect the BLE link and memory budget:
- `ProbeTracker` maintains a bounded table of 32 client probe entries (`heapless::Vec<ClientEntry, 32>`).
- Each entry tracks `client: [u8; 6]`, `ssid: String<32>`, `rssi: i8`, and `last_tick: u32`.
- When full, the least recently heard client entry is evicted.
- A probe is only emitted if:
  1. The `(client, ssid)` pair is previously unseen, OR
  2. The RSSI moves by at least 6 dB, OR
  3. At least 5 seconds have elapsed since the last report.

### 3. RAM & Flash Budget
- **RAM cost**: 32 * 44 bytes = ~1.4 KB RAM on the device.
- **Flash cost**: ~1.2 KB code in `crates/core`.

### 4. Wire Protocol & Web UI
- Telemetry: `TelemetryMessage::ProbeResult { client: [u8; 6], channel: u8, rssi: i8, ssid: String<32> }`.
- Web UI: Clients section in the Scan or Analyze tab displaying:
  - Client MAC with vendor badge (or "Randomized" if locally administered bit `mac[0] & 0x02 != 0`).
  - Probed SSIDs tag list.
  - Signal strength (dBm / bars) and active channel.
