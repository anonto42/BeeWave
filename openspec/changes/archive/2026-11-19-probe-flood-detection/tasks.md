# Tasks

## 1. Protocol & Codec

- [x] 1.1 Add `ProbeFloodAlert { rate: u16, unique_clients: u16 }` to `TelemetryMessage` in `crates/protocol/src/types.rs`
- [x] 1.2 Regenerate golden test vectors with `cargo test -p beewave-protocol --features std --test vectors`
- [x] 1.3 Update TypeScript types in `web/src/shared/protocol/types.ts` and decoder in `web/src/shared/protocol/decode.ts`

## 2. Core Probe Flood Detector

- [x] 2.1 Implement `ProbeFloodDetector` in `crates/core/src/features/probe/flood.rs` and re-export from `crates/core/src/features/probe/mod.rs`
- [x] 2.2 Wire `ProbeFloodDetector` into `Monitoring` service in `crates/core/src/app/monitoring.rs`
- [x] 2.3 Add unit tests for `ProbeFloodDetector` and app integration in `crates/core/src/features/probe/tests.rs` and `crates/core/src/app/tests.rs`

## 3. Web Client & UI

- [x] 3.1 Format `probeFloodAlert` in `web/src/shared/monitor/alerts.ts` with high severity (`alert` / red)
- [x] 3.2 Add mock simulation in `web/src/shared/transport/mock-world.ts` and vitest test in `web/test/shared/monitor/anomaly-rules.test.ts`

## 4. Verification & Documentation

- [x] 4.1 Document probe request flood detection in `docs/MONITORING.md`
- [x] 4.2 Verify all host tests, TypeScript tests, and formatting pass
