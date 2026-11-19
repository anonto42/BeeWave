# Tasks

## 1. Protocol
- [ ] 1.1 Add `DecloakedResult { bssid: [u8; 6], ssid: String<32>, channel: u8, rssi: i8 }` to `TelemetryMessage` in `crates/protocol/src/types.rs`
- [ ] 1.2 Regenerate golden wire vectors (`make vectors`) and mirror decoder in `web/src/shared/protocol/`

## 2. Core Logic & Unit Tests
- [ ] 2.1 Refactor `ScanTable` in `crates/core/src/features/scan/table.rs` to track `had_empty_ssid` in `TableEntry`
- [ ] 2.2 In `ScanTable::observe`, bypass deduplication and return `DecloakedResult` when a hidden BSSID is observed with a non-empty SSID
- [ ] 2.3 Add unit tests in `crates/core/src/features/scan/tests.rs` verifying decloak triggering, deduplication bypass, and prevention of reverting to hidden

## 3. Web Client & UI
- [ ] 3.1 Decode `decloakedResult` in `web/src/shared/protocol/`
- [ ] 3.2 Add `decloaked?: boolean` to `NetworkRecord` and implement `ingestDecloaked` in `web/src/shared/monitor/scan-store.ts`
- [ ] 3.3 Dispatch alerts on `decloakedResult` in `web/src/shared/monitor/alerts.ts` and `AlertStore`
- [ ] 3.4 Render `DECLOAKED` badge in `web/src/features/scanner/template.ts`
- [ ] 3.5 Add unit tests in `web/test` for decloak handling and badge rendering

## 4. Verification & Documentation
- [ ] 4.1 Document hidden SSID decloaking in `docs/MONITORING.md`
- [ ] 4.2 Run `cargo test`, `pnpm test`, and `pnpm run build` to verify clean builds
