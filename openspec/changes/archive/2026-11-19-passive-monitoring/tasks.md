## 1. Protocol
- [x] 1.1 Append commands, telemetry and the `AdClass` / `AnomalyKind` enums; golden vectors; TS codec

## 2. Core
- [x] 2.1 `deauth_detect`, `ble_monitor`, `anomaly` slices with host tests
- [x] 2.2 `app/monitoring.rs` composing them; `SetMonitor` / `GetMonitor`; tick output

## 3. Firmware
- [x] 3.1 Deauth counters in the sniffer callback, drained each second
- [x] 3.2 Passive BLE observer thread (opt-in, duty-cycled) feeding the app task

## 4. Web
- [x] 4.1 Shared monitor kernel (scan store, alert store, session recorder, OUI lookup)
- [x] 4.2 Nine panels, tabs, mock transport coverage, tests (dedup view, thresholds, classes, anomaly rules, export)

## 5. Docs
- [x] 5.1 `docs/MONITORING.md`, USAGE, ARCHITECTURE
