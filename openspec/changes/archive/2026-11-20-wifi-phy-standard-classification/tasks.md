# Tasks: Wi-Fi PHY Standard Classification

## 1. Protocol Definition & Test Vectors

- [x] 1.1 Add `WifiPhyStandard` enum and `TelemetryMessage::PhyResult { bssid: [u8; 6], standard: WifiPhyStandard }` to `crates/protocol/src/types.rs`
- [x] 1.2 Update `crates/protocol/tests/vectors.rs` and run `make vectors` to regenerate `web/test/vectors.json`

## 2. Core Implementation & Frame Parsing

- [x] 2.1 Implement PHY generation parsing in `crates/core/src/features/scan/frame.rs` (Tag 45 HT, Tag 42 ERP, Tag 255 Ext 35 HE, Tags 1 & 50 Supported Rates)
- [x] 2.2 Implement `PhyInfo` and `PhyTracker` bounded table in `crates/core/src/features/scan/phy.rs` and export from `crates/core/src/features/scan/mod.rs`
- [x] 2.3 Wire `phy` into `Observation`, `MonitoringApp`, and `App::on_observation` in `crates/core/src/app/`
- [x] 2.4 Add host unit tests in `crates/core/src/features/scan/tests.rs` and verify with `cargo test -p beewave-core`

## 3. Web Client Protocol & UI Badging

- [x] 3.1 Update TypeScript wire protocol definitions, decoder, and encoder in `web/src/shared/protocol/` (`types.ts`, `decode.ts`, `encode.ts`)
- [x] 3.2 Update `ScanStore` to ingest `phyResult` and persist `standard` on `NetworkRecord` in `web/src/shared/monitor/`
- [x] 3.3 Render color-coded PHY generation badges (`AX`, `N`, `802.11g`, `802.11b`) in `web/src/features/scanner/template.ts`
- [x] 3.4 Update mock world and tests in `web/test/` and verify with `pnpm --dir web test` and `pnpm --dir web build`

## 4. Documentation & Verification

- [x] 4.1 Document PHY standard classification in `docs/MONITORING.md`
- [x] 4.2 Verify complete workspace checks (`cargo fmt`, `cargo check`, `pnpm --dir web test`, `pnpm --dir web build`)
