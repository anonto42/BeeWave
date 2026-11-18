# Tasks

## 1. Protocol
- [x] 1.1 Append `ProbeResult { client: [u8; 6], channel: u8, rssi: i8, ssid: String<32> }` to `TelemetryMessage` in `crates/protocol/src/types.rs`
- [x] 1.2 Update golden wire vectors (`make vectors`) and mirror codec in `web/src/shared/protocol/`

## 2. Core Logic & Unit Tests
- [x] 2.1 Implement `parse_probe_request` in `crates/core/src/features/probe/parser.rs` with zero-allocation slice parsing
- [x] 2.2 Implement `ProbeTracker` bounded deduplication table in `crates/core/src/features/probe/tracker.rs`
- [x] 2.3 Add host unit tests in `crates/core/src/features/probe/tests.rs` for directed, wildcard, and malformed frames
- [x] 2.4 Wire probe frame reception and deduplication into `App` service and monitoring pipeline

## 3. Web Client
- [x] 3.1 Add client probe tracking and PNL correlation in `web/src/shared/monitor/`
- [x] 3.2 Render Discovered Clients panel with vendor resolution and probed SSIDs
- [x] 3.3 Add mock probe request simulation to `web/src/shared/transport/mock-world.ts` and unit tests

## 4. Verification & Documentation
- [x] 4.1 Update `docs/MONITORING.md` with client probe sniffing documentation
- [x] 4.2 Verify entire project with `make check`
