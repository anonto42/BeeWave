# Tasks: Beacon Timestamp (TSF) Rollback Detection

## 1. Protocol Definition & Test Vectors

- [x] 1.1 Add `TsfRollback` variant to `AnomalyKind` in `crates/protocol/src/types.rs`
- [x] 1.2 Update `crates/protocol/tests/vectors.rs` and regenerate vectors with `make vectors`

## 2. Core Implementation & Anomaly Detection

- [x] 2.1 Extract 64-bit TSF timestamp from beacon frames in `crates/core/src/features/scan/frame.rs` and add `tsf: u64` to `Observation` in `crates/core/src/features/scan/types.rs`
- [x] 2.2 Update `KINDS = 7` and `kind_index` in `crates/core/src/features/anomaly/types.rs`
- [x] 2.3 Implement TSF rollback detection in `crates/core/src/features/anomaly/detector.rs`
- [x] 2.4 Add unit tests for TSF extraction and rollback detection in `crates/core/src/features/anomaly/tests.rs` and verify with `cargo test -p beewave-core`

## 3. Web Client Alert Formatting & Mock

- [x] 3.1 Add `"tsfRollback"` to `ANOMALY_KINDS` in `web/src/shared/protocol/monitor.ts`
- [x] 3.2 Format `[ROGUE AP DETECTED]` alert in `web/src/shared/monitor/alerts.ts`
- [x] 3.3 Add simulation in `web/src/shared/transport/mock-world.ts` and test cases in `web/test/`
- [x] 3.4 Verify with `pnpm --dir web test` and `pnpm --dir web build`

## 4. Documentation & Verification

- [x] 4.1 Document TSF rollback detection in `docs/MONITORING.md`
- [x] 4.2 Verify complete workspace checks (`cargo fmt`, `cargo check`, `pnpm --dir web test`, `pnpm --dir web build`)
