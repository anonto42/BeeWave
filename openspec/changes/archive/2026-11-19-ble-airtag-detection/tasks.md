# Tasks

## 1. Protocol & Codec

- [x] 1.1 Add `FindMy` to `AdClass` and `find_my: u16` to `BleStats` in `crates/protocol/src/types.rs`
- [x] 1.2 Regenerate golden vectors with `cargo test -p beewave-protocol --features std --test vectors`
- [x] 1.3 Mirror in `web/src/shared/protocol/monitor.ts`, `types.ts`, and `decode.ts` and verify with vitest

## 2. Core Implementation

- [x] 2.1 Add `APPLE_FIND_MY` (0x12) classification in `crates/core/src/features/ble_monitor/classify.rs`
- [x] 2.2 Update `CLASSES = 5` and `ALL_CLASSES` in `crates/core/src/features/ble_monitor/types.rs`
- [x] 2.3 Update `crates/core/src/app/monitoring.rs` to extract `find_my` in `BleStats`
- [x] 2.4 Add unit tests for Find My advertisement parsing, rolling statistics, and spike alerts in `crates/core/src/features/ble_monitor/tests.rs`

## 3. Web Client & UI

- [x] 3.1 Update `MonitorState` in `web/src/shared/monitor/monitor-state.ts` to track `findMy`
- [x] 3.2 Update `LABEL` and alert text in `web/src/features/ble-monitor/template.ts` and `alerts.ts`
- [x] 3.3 Add mock Find My advertisements in `web/src/shared/transport/mock-world.ts` and test with vitest

## 4. Verification

- [x] 4.1 Document Find My / AirTag detection in `docs/MONITORING.md`
- [x] 4.2 Verify all host tests, web tests, and formatting pass
