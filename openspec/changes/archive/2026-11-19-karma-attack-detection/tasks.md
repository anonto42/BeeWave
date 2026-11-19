# Tasks

## 1. Protocol & Codec

- [x] 1.1 Add `KarmaSsidChange` variant to `AnomalyKind` in `crates/protocol/src/types.rs`
- [x] 1.2 Regenerate golden test vectors with `cargo test -p beewave-protocol --features std --test vectors`
- [x] 1.3 Mirror `karmaSsidChange` in `web/src/shared/protocol/monitor.ts`

## 2. Core Anomaly Detector

- [x] 2.1 Update `KINDS = 6` and `kind_index` in `crates/core/src/features/anomaly/types.rs`
- [x] 2.2 Implement KARMA / Rogue AP SSID mutation detection in `crates/core/src/features/anomaly/detector.rs`
- [x] 2.3 Add unit tests covering KARMA SSID change and hidden network preservation in `crates/core/src/features/anomaly/tests.rs`

## 3. Web Client & UI

- [x] 3.1 Format `karmaSsidChange` alert in `web/src/shared/monitor/alerts.ts` with high severity (`alert` / red)
- [x] 3.2 Add mock simulation in `web/src/shared/transport/mock-world.ts` and vitest tests in `web/test/shared/monitor/anomaly-rules.test.ts`

## 4. Verification & Documentation

- [x] 4.1 Document KARMA rogue AP SSID change detection in `docs/MONITORING.md`
- [x] 4.2 Verify all host tests, TypeScript tests, and formatting pass
