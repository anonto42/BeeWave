# Tasks

## 1. Protocol
- [x] 1.1 Append `AssocResult { bssid: [u8; 6], client: [u8; 6], channel: u8, rssi: i8 }` to `TelemetryMessage` in `crates/protocol/src/types.rs`
- [x] 1.2 Update golden wire vectors (`make vectors`) and mirror codec in `web/src/shared/protocol/`

## 2. Core Logic & Unit Tests
- [x] 2.1 Implement `parse_data_association` in `crates/core/src/features/assoc/parser.rs` with ToDS/FromDS address resolution
- [x] 2.2 Implement bounded `AssocTracker` in `crates/core/src/features/assoc/tracker.rs` with LRU eviction and deduplication
- [x] 2.3 Add host unit tests in `crates/core/src/features/assoc/tests.rs` for ToDS, FromDS, multicast/broadcast rejection, and deduplication
- [x] 2.4 Wire `assoc` observation routing and deduplication into `App` service and monitoring pipeline

## 3. Firmware Adapter
- [x] 3.1 Extract data frame associations in `firmware/src/adapters/wifi_radio/callback.rs` and install queue sender
- [x] 3.2 Wire `ASSOC_DEPTH` queue in `firmware/src/runtime/channels.rs`, `app_task.rs`, and `main.rs`

## 4. Web Client & UI
- [x] 4.1 Create `AssocStore` in `web/src/shared/monitor/assoc-store.ts` and integrate with `MonitorKernel`
- [x] 4.2 Render connected stations list (MAC, vendor, RSSI, last active) under target APs in Scan tab
- [x] 4.3 Add mock association traffic simulation in `web/src/shared/transport/mock-world.ts` and unit tests

## 5. Verification & Documentation
- [x] 5.1 Document client association tracking in `docs/MONITORING.md`
- [x] 5.2 Verify entire project with `cargo test`, `pnpm test`, and `pnpm run build`
