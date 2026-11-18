# Proposal

## Why
While BeeWave can passively discover Access Points (via Beacons) and unassociated searching client stations (via Probe Requests), security auditors and site surveyors currently have no visibility into which client devices are actively connected to which Access Points. Monitoring 802.11 Data frame headers passively exposes live client-to-AP associations without transmitting any packets or disrupting legitimate wireless traffic.

## What Changes
- Add `AssocResult` telemetry message to `crates/protocol` carrying AP BSSID, Client MAC, Channel, and RSSI.
- Add zero-allocation 802.11 Data and QoS Data frame parser in `crates/core/src/features/assoc/parser.rs` extracting client and BSSID from ToDS/FromDS address fields.
- Add bounded `AssocTracker` in `crates/core/src/features/assoc/tracker.rs` with LRU eviction and 6 dB / 5s deduplication.
- Wire data frame processing in firmware promiscuous sniffer callback and app task queue.
- Add `AssocStore` in `web/src/shared/monitor/assoc-store.ts` to map connected stations to Access Points.
- Display connected client counts and station list (MAC, Vendor, RSSI) under Access Points in the Scan/Target tab.
- Correlate connected stations in the Discovered Clients panel.

## Capabilities

### Modified Capabilities
- `device-core`: Add passive 802.11 Data frame association extraction and bounded `AssocTracker`.
- `wire-protocol`: Add `AssocResult` variant to `TelemetryMessage`.
- `web-client`: Add client-to-AP association tracking and render connected stations under Access Point targets.

## Impact
- `crates/protocol`: Appends `AssocResult` variant to `TelemetryMessage` (non-breaking, golden vectors regenerated).
- `crates/core`: New `#![no_std]` feature slice `assoc` (parser, tracker, tests), wired to `App` monitoring pipeline.
- `firmware`: Promiscuous RX callback dispatches data frame headers to a bounded `assoc_tx` channel.
- `web`: `web/src/shared/protocol/` decodes `AssocResult`, `AssocStore` collects associations, UI mounts connected clients in Target details.
