# Testing

## Host tests (no hardware)
`make check` runs everything that does not need the board: format, Rust unit tests (`crates/protocol`, `crates/core`),
the `no_std` build, the web tests (codec vs the Rust golden vectors, auth, OTA uploader) and the OpenSpec validation.
`make fw-build` and `make fw-build FEATURES=coex_debug` only prove the firmware compiles.

## Hardware preparation
1. `make fw-key` and `make fw-ota-key` (once; they create the untracked device key and OTA signing key).
2. `make fw-run` (builds with both keys, flashes, opens the monitor). Add `FEATURES=coex_debug` for latency logging.
3. Open the PWA on the phone, paste the key printed by `make fw-key` into **DEVICE KEY**, tap **SAVE KEY**.

## Smoke test (about 10 minutes)
Tick each line. The exact texts are what the UI shows.
- [ ] Serial log shows `beewave up`, no panic and no reboot loop (`make fw-monitor`).
- [ ] The Chrome chooser lists `BeeWave`; **CONNECT** succeeds.
- [ ] **Right key:** the DEVICE KEY panel goes `authenticating...` then `authenticated`, and the STATUS panel starts updating every second.
- [ ] **Wrong key:** SAVE KEY with a different 64-hex key, CONNECT: the panel shows `rejected: wrong key?`, STATUS stays empty, and pressing START SCAN only adds `event 4: not authenticated` to the LOG.
- [ ] **No key:** press **FORGET**, CONNECT: the panel shows `no key saved`, STATUS stays empty, and START SCAN adds `event 4: not authenticated` to the LOG.
- [ ] After a disconnect and reconnect the session is gone: with the right key saved, it authenticates again by itself.
- [ ] START SCAN lists nearby networks; STOP stops new rows.
- [ ] SET CHANNEL pins the channel shown in STATUS; AUTO makes it change.
- [ ] SAVE TO DEVICE, power-cycle the board, reconnect: the settings are restored.
- [ ] REBOOT from the UI, reconnect: it authenticates again automatically.
- [ ] Firmware update with `dist/beewave-app.bin` + `.bin.sig` (from `make fw-image`) completes and the new image boots.
- [ ] Firmware update with a `.sig` from another build is refused at once with `event 2: ota refused: bad signature`.

## Coexistence measurement
Wi-Fi sniffing and BLE share one radio. To measure notification latency while hopping:
1. `make fw-run FEATURES=coex_debug`, connect and authenticate, START SCAN with channel AUTO.
2. Every 10 s the serial log prints `coex queue: n=.. p50<=..us p99<=..us max=..us` (time a message waited
   for the BLE task) and `coex notify: ...` (time the notify call took). Percentiles are bucket upper edges.
3. Watch BLE LOST in the STATUS panel for the same period.
4. Target: queue p99 at or under 50000 us and BLE LOST not growing over 10 minutes.
5. If it misses, change one knob at a time and re-measure: `DWELL` and `HOP_ORDER` in
   `firmware/src/adapters/wifi_radio/hopper.rs`, `REQUEST_CONN_PARAMS` and the interval/latency constants in
   `firmware/src/adapters/ble_gatt/tuning.rs`, queue depths in `firmware/src/runtime/channels.rs`.

## Acceptance sign-off
Record the full hardware results (security checks and performance targets) in `docs/ACCEPTANCE.md`.
