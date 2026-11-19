# Tasks

## 1. Protocol & Codec

- [x] 1.1 Add `SecurityDowngrade` variant to `AnomalyKind` in `crates/protocol/src/types.rs`
- [x] 1.2 Regenerate golden vectors with `cargo test -p beewave-protocol --features std --test vectors`
- [x] 1.3 Mirror `securityDowngrade` in `web/src/shared/protocol/monitor.ts` and test with vitest

## 2. Core Implementation

- [x] 2.1 Update `KINDS = 5` and `kind_index` in `crates/core/src/features/anomaly/types.rs`
- [x] 2.2 Store `security_proto` in `Entry` and implement Evil Twin security downgrade detection in `crates/core/src/features/anomaly/detector.rs`
- [x] 2.3 Add unit tests covering Evil Twin downgrade detection and legitimate multi-AP networks in `crates/core/src/features/anomaly/tests.rs`

## 3. Web Client & UI

- [x] 3.1 Format `securityDowngrade` alerts in `web/src/shared/monitor/alerts.ts` with high severity
- [x] 3.2 Add mock simulation in `web/src/shared/transport/mock-world.ts` and unit test in `web/test/shared/monitor/anomaly-rules.test.ts`

## 4. Verification

- [x] 4.1 Document Evil Twin security downgrade detection in `docs/MONITORING.md`
- [x] 4.2 Verify all host tests, TypeScript tests, and formatting pass
