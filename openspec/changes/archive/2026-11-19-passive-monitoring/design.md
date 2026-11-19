## Data flow

```
Wi-Fi promiscuous callback --beacon--> frames queue --> App::on_observation --> ScanTable (dedup) + AnomalyDetector
                           --deauth--> atomic per-BSSID counters (8 slots) --drained each second--> App::on_deauth_second
NimBLE passive scan (opt-in) --classify(payload)--> ble queue --> App::on_ble_adv
App::on_tick (1 Hz): Status, DeauthAlert (>= threshold), BleStats every 10 s, BleSpike
```

## Bounds (heapless only)

- Deauth: 8 lock-free slots in the adapter; the core sees at most 8 (bssid, count) pairs per second and
  alerts the top 4.
- BLE: four 60-bucket u16 rings (480 B), a per-class baseline (EMA), spike = count above
  `max(baseline * factor, baseline + 20)` after a 60 s warm-up.
- Anomaly: 64 BSSID entries (about 20 B each), evicting the least recently seen; a 30 s cooldown per
  BSSID and kind bounds alert rate. One *cycle* is 4 ticks (about one 13-channel lap).

## Configuration

`SetMonitor { deauth_threshold (0 = off), ble_enabled, ble_spike_factor }` is validated in the core and
answered with `MonitorConfig`. It is not persisted. The app task mirrors `ble_enabled` into the BLE
observer's flag.

## Web

`shared/monitor` holds the stores (scan history with cycles and snapshots, alerts, session recorder) that
the context feeds from telemetry, so panels in hidden tabs lose nothing. Panels are isolated feature
slices that read those stores; the watchlist is a pure web rule set over the same stream.
