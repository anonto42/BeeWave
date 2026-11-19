# Tasks

## 1. Protocol
- [x] 1.1 Add `BleFloodAlert { rate: u16, class: AdClass }` to `TelemetryMessage` in `crates/protocol/src/types.rs`
- [x] 1.2 Regenerate golden wire vectors (`make vectors`) and mirror decoder in `web/src/shared/protocol/`

## 2. Core Logic & Unit Tests
- [x] 2.1 Implement `BleFloodDetector` in `crates/core/src/features/ble_monitor/flood.rs`
- [x] 2.2 Wire `BleFloodDetector` into `Monitoring` on `on_ble_adv` and 1 Hz `on_tick` in `crates/core/src/app/monitoring.rs`
- [x] 2.3 Add unit tests in `crates/core/src/features/ble_monitor/tests.rs` for threshold, dominant class detection, and cooldown

## 3. Web Client & UI
- [x] 3.1 Decode `bleFloodAlert` in `web/src/shared/protocol/`
- [x] 3.2 Add `"bleFlood"` to `AlertSource` in `web/src/shared/monitor/types.ts`
- [x] 3.3 Format `bleFloodAlert` in `web/src/shared/monitor/alerts.ts` and dispatch to `AlertStore`
- [x] 3.4 Add mock simulation in `web/src/shared/transport/mock-world.ts` and add unit tests in `web/test`

## 4. Verification & Documentation
- [x] 4.1 Document BLE flood attack detection in `docs/MONITORING.md`
- [x] 4.2 Verify entire project with `cargo test`, `pnpm test`, and `pnpm run build`
