# Proposal

## Why

Currently, BeeWave sniffs 802.11 beacon frames and probe responses to identify Access Points. However, client devices (smartphones, laptops, IoT stations) actively transmit 802.11 Probe Request frames (`subtype 0x40`) searching for known Wi-Fi networks in their Preferred Network Lists (PNL) or probing for any available AP (wildcard probe). Adding passive probe request sniffing allows users to discover nearby client devices, observe the networks those devices are seeking, detect unassociated devices, and determine whether clients are using randomized MAC addresses — all completely passively without transmitting any RF frames.

## What Changes

- **Core parsing (`crates/core/src/features/probe`)**:
  - Add zero-allocation parser `parse_probe_request` to parse 802.11 Probe Request frames (subtype `0x40`).
  - Extract client transmitter MAC address (`[u8; 6]`), requested SSID (`String<32>`, empty for wildcard), RSSI (`i8`), and channel (`u8`).
  - Add `ProbeTracker` deduplication table bounded with `heapless` to prevent flooding the BLE link (re-reporting only on new client/SSID pairs or significant RSSI moves).
- **Wire protocol (`crates/protocol`)**:
  - Append a new telemetry variant `ProbeResult { client: [u8; 6], channel: u8, rssi: i8, ssid: String<32> }` to `TelemetryMessage`. Protocol version remains 1.
- **Core integration (`crates/core/src/app`)**:
  - Wire probe request parsing into the Wi-Fi promiscuous reception pipeline and App service.
- **Firmware adapter (`firmware/src/adapters/wifi_radio`)**:
  - Forward `0x40` (Probe Request) frames from the sniffer callback alongside existing beacon frames.
- **Web client (`web/src`)**:
  - Mirror `ProbeResult` in protocol codec.
  - Add client probe tracking in `shared/monitor/` (correlating client MAC, vendor OUI, probed SSIDs).
  - Add a "Clients" view / panel in the web UI showing discovered client devices, randomized vs vendor MACs, RSSI, and their probed network list.

## Non-Goals & Invariants

- **Strictly Receive-Only**: BeeWave does not transmit probe responses or inject any 802.11 frames.
- **Zero Allocation**: Pure `no_std`, heapless collections on the device path.
- **Memory Bounded**: Deduplication table uses fixed capacity (`heapless::Vec`) with deterministic LRU eviction.

## Capabilities

### Modified Capabilities
- `device-core`: Passive extraction and bounded deduplication of 802.11 probe requests.
- `wire-protocol`: Telemetry variant `ProbeResult` appended to `TelemetryMessage`.
- `web-client`: Discovered client device table displaying client MACs, vendor, and probed SSIDs.

## Impact

- `crates/protocol`: Appends `ProbeResult` to `TelemetryMessage`; golden vectors regenerated.
- `crates/core`: New feature slice `features/probe/` with unit tests; wired into `App`.
- `firmware`: Sniffer callback admits `0x40` management frames.
- `web`: `shared/protocol/` mirrors wire codec; new or extended UI view for clients.
- Docs: Updated in `docs/MONITORING.md`.
