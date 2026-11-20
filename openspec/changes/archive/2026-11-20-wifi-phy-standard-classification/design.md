# Design: Wi-Fi PHY Standard Classification

## Context

See `proposal.md` for background and motivation.
Management frames broadcast Information Elements detailing supported data rates and physical layer capabilities:
- **Element 45 (0x2D)**: HT Capabilities (High Throughput / 802.11n Wi-Fi 4).
- **Element 42 (0x2A)**: ERP Information (Extended Rate PHY / 802.11g).
- **Element 255 (0xFF)**: Extension Element with Ext ID 35 (`0x23`), advertising HE Capabilities (High Efficiency / 802.11ax Wi-Fi 6).
- **Elements 1 & 50**: Supported Rates and Extended Supported Rates; rate values $> 22$ (11 Mbps) indicate 802.11g OFDM rates; values $\le 22$ indicate legacy 802.11b DSSS rates.

## Goals / Non-Goals

**Goals:**
- Zero-allocation parsing in `crates/core` with `#![no_std]`.
- Bounded LRU deduplication (`PhyTracker`, 32 entries) so `PhyResult` is sent only once per BSSID unless changed.
- Appending `PhyResult` to `TelemetryMessage` without breaking existing variant encodings.
- Direct badging in the PWA Scanner cards (`AX`, `N`, `802.11g`, `802.11b`).

**Non-Goals:**
- Demodulating 5 GHz channels (the ESP32-WROOM hardware radio is 2.4 GHz only).
- Parsing MCS index maps or detailed HT spatial stream fields (only PHY generation classification is required).

## Decisions

### 1. Dedicated `PhyResult` Telemetry Message Variant
- **Decision**: Append `PhyResult { bssid: [u8; 6], standard: WifiPhyStandard }` to `TelemetryMessage` (tag 32).
- **Rationale**: Keeps existing telemetry variants untouched; 7-byte body size ensures minimal BLE overhead.

### 2. Element Precedence Order
- **Decision**: In `crates/core/src/features/scan/frame.rs`:
  1. If HE Capabilities (Tag 255 Ext 35) present $\to$ `Ax` (Wi-Fi 6)
  2. Else if HT Capabilities (Tag 45) present $\to$ `N` (Wi-Fi 4)
  3. Else if ERP (Tag 42) present or rates $> 22$ $\to$ `G` (802.11g)
  4. Else if basic rates present $\to$ `B` (802.11b)
  5. Else $\to$ `Unknown`
- **Rationale**: Follows standard 802.11 generation evolution hierarchy.

### 3. RAM & Flash Footprint
- `PhyInfo` is 7 bytes (`[u8; 6]` + `WifiPhyStandard`).
- `PhyTracker` holds up to 32 entries $\to$ $32 \times 7 = 224$ bytes of stack memory. Zero dynamic heap allocation.

## Risks / Trade-offs

- **[Risk] Corrupted or truncated elements** $\to$ *Mitigation*: The element loop bounds-checks lengths and defaults safely to `Unknown` if frames are truncated.
- **[Risk] BLE saturation** $\to$ *Mitigation*: `PhyTracker` deduplicates reports and only re-emits if the standard changes.
