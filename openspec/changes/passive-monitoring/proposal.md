## Why

The PWA only lists beacons. Owners want to see what the radio environment is doing: who is deauthenticating
whom, whether BLE spam is in the air, which networks behave oddly, and how scans compare over time. All of it
can be learned by *listening*; nothing needs to be transmitted.

## What Changes

- **Receive-only monitors on the device**, each a core slice with host tests:
  - `deauth_detect`: counts deauthentication frames (subtype 0xC0) per BSSID per second from the existing
    promiscuous sniffer and raises `DeauthAlert` at a configurable threshold (default 10/s).
  - `ble_monitor`: classifies passively received BLE advertisements (Fast Pair, iBeacon, AirPods-style
    proximity, generic), counts per minute, raises `BleSpike` when a class exceeds its learned baseline.
    **Off by default**, enabled from the UI.
  - `anomaly`: `AnomalyAlert` for a known SSID on a new BSSID, a BSSID on two channels within a minute, an
    RSSI jump over 20 dB between cycles, and a network back after more than 5 missed cycles.
- **Wire protocol (append-only, version stays 1):** commands `GetMonitor`, `SetMonitor`; telemetry
  `DeauthAlert`, `BleStats`, `BleSpike`, `AnomalyAlert`, `MonitorConfig`; enums `AdClass`, `AnomalyKind`.
- **Web (phone only):** expanded results table, channel occupancy and utilisation, RSSI history with a
  delta view, deauth / BLE / anomaly panels, scan comparison with JSON export, a watchlist kept in
  `localStorage`, and a session export (JSON and CSV). The mock transport drives all of them.

## Non-goals

- No transmit path of any kind: no deauth, no probe, no beacon, no BLE advertisement or scan request
  (the BLE scan is passive). No change to `REQUEST_CONN_PARAMS` or bonding.
- No device-side watchlist: it lives on the phone (the owner's call), so there is no core `watchlist` slice.
- No persistence of the monitor configuration on the device (RAM only, defaults at boot).

## Risks

- BLE scanning shares the radio with the GATT link and the Wi-Fi sniffer. It is duty-cycled (25%) and off
  until the user enables it; link quality must be checked on the bench with it on (`BLE LOST`).
- The channel a beacon is *received* on can differ from its home channel through adjacent-channel leakage, so
  the two-channels rule can fire on a healthy AP; it is rate-limited and labelled as a hint.

## Impact

`crates/protocol`, `crates/core` (three slices + `app/monitoring.rs`), `firmware` (deauth counters in the
sniffer callback, a passive BLE observer thread, app-task wiring), `web` (shared monitor kernel, nine panels,
mock transport, tests). RAM: about 2 KB of bounded tables on the device.
