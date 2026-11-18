# Tasks

## 1. Protocol
- [ ] 1.1 Add `BeaconFloodAlert { rate: u16, unique_bssids: u16 }` to `TelemetryMessage` in `crates/protocol/src/types.rs`
- [ ] 1.2 Regenerate golden wire vectors (`make vectors`) and mirror decoder in `web/src/shared/protocol/`

## 2. Core Logic & Unit Tests
- [ ] 2.1 Implement `BeaconFloodDetector` in `crates/core/src/features/scan/flood.rs`
- [ ] 2.2 Wire `BeaconFloodDetector` into `Monitoring` on observations and 1 Hz tick in `crates/core/src/app/monitoring.rs`
- [ ] 2.3 Add unit tests in `crates/core/src/features/scan/tests.rs` and `crates/core/src/app/tests.rs` for flood threshold, BSSID uniqueness, and cooldown

## 3. Web Client & UI
- [ ] 3.1 Decode `beaconFloodAlert` and format alerts in `web/src/shared/monitor/alerts.ts`
- [ ] 3.2 Add mock beacon flood simulation in `web/src/shared/transport/mock-world.ts` and add unit tests in `web/test`

## 4. Verification & Documentation
- [ ] 4.1 Document beacon flood attack detection in `docs/MONITORING.md`
- [ ] 4.2 Verify entire project with `cargo test`, `pnpm test`, and `pnpm run build`
