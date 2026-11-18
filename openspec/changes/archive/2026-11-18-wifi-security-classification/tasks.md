# Tasks

## 1. Protocol
- [x] 1.1 Add `WifiSecurityProtocol`, `WifiPmfMode`, and `SecurityResult` variant to `TelemetryMessage` in `crates/protocol/src/types.rs`
- [x] 1.2 Regenerate golden wire vectors (`make vectors`) and mirror decoder/types in `web/src/shared/protocol/`

## 2. Core Logic & Unit Tests
- [x] 2.1 Implement zero-allocation security element parser in `crates/core/src/features/scan/security.rs` and `SecurityTracker` bounded deduplication table
- [x] 2.2 Wire security parsing into `parse_beacon` and observation routing in `App::on_observation`
- [x] 2.3 Add comprehensive unit tests in `crates/core/src/features/scan/tests.rs` for Open, WEP, WPA2, WPA3, Transition, OWE, PMF flags, and deduplication

## 3. Web Client & UI
- [x] 3.1 Ingest `SecurityResult` in `ScanStore` and map security posture to `NetworkRecord`
- [x] 3.2 Display security badges (`OPEN`, `WEP`, `WPA2`, `WPA3`, `OWE`, `PMF`) in Scanner and Target panels
- [x] 3.3 Add mock security data in `web/src/shared/transport/mock-world.ts` and add unit tests in `web/test`

## 4. Verification & Documentation
- [x] 4.1 Document Wi-Fi security suite and PMF classification in `docs/MONITORING.md`
- [x] 4.2 Verify entire project with `cargo test`, `pnpm test`, and `pnpm run build`
