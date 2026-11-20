# Proposal: Wi-Fi PHY Standard & Generation Classification (802.11b / 802.11g / 802.11n Wi-Fi 4 / 802.11ax Wi-Fi 6)

## Why

Currently, BeeWave's 802.11 beacon parser extracts SSID, DS channel, RSN security, and WPS information, but ignores physical layer (PHY) capability Information Elements: HT Capabilities (Element 45 / 802.11n Wi-Fi 4), ERP Information (Element 42 / 802.11g), HE Capabilities (Element 255 Ext 35 / 802.11ax Wi-Fi 6), and Supported Rates (Elements 1 & 50).

In wireless security auditing and site surveys, knowing an Access Point's physical layer generation is critical: legacy 802.11b networks introduce serious airtime degradation and lack modern protection mechanisms, while modern 802.11n (Wi-Fi 4) and 802.11ax (Wi-Fi 6) APs support high-throughput rates. Classifying and badging the physical generation of each Access Point provides instant operational visibility.

## What Changes

- **Protocol**:
  - Add `WifiPhyStandard` enum (`Unknown`, `B`, `G`, `N`, `Ax`).
  - Add `PhyResult { bssid: [u8; 6], standard: WifiPhyStandard }` variant to `TelemetryMessage`.
- **Core**:
  - Parse HT Capabilities (Tag 45), ERP Information (Tag 42), HE Capabilities (Tag 255 Ext 35), and Supported Rates (Tags 1 & 50) zero-allocation in `#![no_std]`.
  - Add `PhyInfo` and `PhyTracker` bounded deduplication table (32 entries) to emit `PhyResult` only when first observed or if the PHY standard changes.
  - Wire into `Observation`, `MonitoringApp`, and `App::on_observation`.
- **Web Client**:
  - Update TypeScript wire codec (`types.ts`, `decode.ts`, `encode.ts`) for `phyResult`.
  - Ingest `phyResult` into `ScanStore`, storing `standard` on `NetworkRecord`.
  - Display color-coded generation badges (`AX`, `N`, `802.11g`, `802.11b`) in the Scanner panel cards.
  - Update `MockWorld` and test suites.

## Capabilities

### Modified Capabilities
- `wire-protocol`: Add `WifiPhyStandard` enum and `PhyResult` variant to `TelemetryMessage`.
- `device-core`: Parse 802.11 PHY generation Information Elements and deduplicate via `PhyTracker`.
- `web-client`: Ingest `PhyResult` into `ScanStore` and render generation badges on Scanner network cards.

## Impact

- `crates/protocol`: Extends `types.rs` with `WifiPhyStandard` and `PhyResult`. Wire format addition; regenerated vectors in `web/test/vectors.json`.
- `crates/core`: Zero-allocation frame parsing in `scan/frame.rs` and bounded tracking in `scan/phy.rs`.
- `web`: Zero external dependency additions; pnpm-only; full vitest and TypeScript test coverage.
