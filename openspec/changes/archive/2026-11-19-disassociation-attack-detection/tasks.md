# Tasks

## 1. Protocol & Codec

- [x] 1.1 Append `DisassocAlert { bssid: [u8; 6], count: u16 }` to `TelemetryMessage` in `crates/protocol/src/types.rs`
- [x] 1.2 Add golden vector test in `crates/protocol/tests/vectors.rs` and update `web/test/vectors.json`
- [x] 1.3 Mirror `DisassocAlert` decoder in `web/src/shared/protocol/types.ts` and `web/src/shared/protocol/decode.ts` and verify with `web/test/shared/protocol.test.ts`

## 2. Core Implementation

- [x] 2.1 Add `on_disassoc_window` handling in `crates/core/src/features/deauth_detect/` and wire into `crates/core/src/app/monitoring.rs` and `crates/core/src/app/service.rs`
- [x] 2.2 Add unit tests in `crates/core/src/app/tests.rs` verifying threshold triggers, multi-BSSID limits, and authentication gating

## 3. Firmware Integration

- [x] 3.1 Add `DISASSOC_FC0` (`0xA0`) recognition in `firmware/src/adapters/wifi_radio/callback.rs` and separate or unified lock-free counting in `firmware/src/adapters/wifi_radio/deauth.rs`
- [x] 3.2 Wire disassociation drain into `firmware/src/runtime/app_task.rs`

## 4. Web Client & UI

- [x] 4.1 Ingest `DisassocAlert` in `web/src/shared/monitor/alerts.ts` and `monitor-state.ts`, formatting `[DISASSOC FLOOD]` alerts
- [x] 4.2 Add mock disassociation flood in `web/src/shared/transport/mock-world.ts` and test in `web/test/shared/monitor/anomaly-rules.test.ts`

## 5. Documentation & Verification

- [x] 5.1 Document 802.11 disassociation attack detection in `docs/MONITORING.md`
- [x] 5.2 Run `make check` and verify host tests, TypeScript tests, and formatting pass
