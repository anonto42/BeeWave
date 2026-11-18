# Tasks

## 1. Protocol
- [x] 1.1 Append `WpsResult { bssid: [u8; 6], version: u8, state: u8, locked: bool, selected_registrar: bool, device_name: String<32>, manufacturer: String<32>, model_name: String<32> }` to `TelemetryMessage` in `crates/protocol/src/types.rs`
- [x] 1.2 Update golden wire vectors (`make vectors`) and mirror codec in `web/src/shared/protocol/`

## 2. Core Logic & Unit Tests
- [x] 2.1 Implement WPS IE TLV parser in `crates/core/src/features/scan/wps.rs` extracting version, state, locked, registrar, device, manufacturer, and model
- [x] 2.2 Wire WPS parsing into `parse_beacon` and observation routing with bounded `WpsTracker` deduplication
- [x] 2.3 Add host unit tests in `crates/core/src/features/scan/tests.rs` for valid WPS, locked status, malformed TLVs, and deduplication

## 3. Web Client & UI
- [x] 3.1 Store WPS records in `ScanStore` and map to Access Point records
- [x] 3.2 Display WPS badges (version, locked/open, push button) and device details in Scanner and Target views
- [x] 3.3 Add mock WPS simulation in `web/src/shared/transport/mock-world.ts` and unit tests

## 4. Verification & Documentation
- [x] 4.1 Document WPS Information Element parsing and device fingerprinting in `docs/MONITORING.md`
- [x] 4.2 Verify entire project with `cargo test`, `pnpm test`, and `pnpm run build`
