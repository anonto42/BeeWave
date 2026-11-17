# Architecture

Hexagonal (ports & adapters) inside, feature slices on the outside. Same rules in Rust and TypeScript.

```
crates/protocol   wire types + postcard codec            (no_std, depends on nothing)
crates/core       domain + ports + app service           (no_std, depends on protocol only)
firmware/         ESP-IDF adapters + composition root    (depends on core; ONLY crate with esp-idf)
web/              PWA, feature slices                    (mirrors the protocol in shared/protocol/; Transport = the port)
```

## Dependency rule
`adapters -> core -> protocol`. Never the reverse. `core` has no `esp-idf`, no threads, no heap allocation
(heapless only) so every rule is unit-tested on the host with `make test`.

## Rust core
```
crates/core/src/
  ports/                 one file per trait the outside world implements (TelemetrySink, RadioDriver, System, OtaWriter, SettingsStore)
  app/                   App: routes driving input (commands, frames, ticks) to slices, publishes via the sink; DeviceKeys
  features/<slice>/      state + rules + tests for ONE capability: radio, scan, status, ota, settings, auth
```
Slices depend on `ports` and `protocol`, never on each other. `App` is the only place that composes them.

### Add a feature (Rust)
1. Wire message(s) in `crates/protocol` -> `make vectors` -> mirror in `web/src/shared/protocol/` (types, encode, decode).
2. `crates/core/src/features/<name>/mod.rs` with its state machine and tests; new port in `ports/<name>.rs` if it needs hardware.
3. Route it in `App::on_command` / `on_observation` / `on_tick` (`app/service.rs`).
4. Implement the port in `firmware/src/adapters/<name>/`; construct it in `firmware/src/main.rs` (composition root).

## Firmware
`adapters/*` = every ESP-IDF call. `runtime/` = threads + bounded channels (depths are constants in `runtime/channels.rs`); `config.rs` = build-time keys. `main.rs` = wiring only.
Driving adapters push `Event`s into one bounded channel; the `app` task owns `App` (no locks needed).
Telemetry is lossy by contract: `BleSink::publish` never blocks, drops are counted and reported in `Status.dropped`.

## Passive monitoring
Three receive-only core slices (`deauth_detect`, `ble_monitor`, `anomaly`) are composed in `app/monitoring.rs`. The sniffer callback counts
deauth frames into lock-free counters (`wifi_radio/deauth.rs`), a passive NimBLE observer (`ble_observer/`, opt-in) feeds classified
advertisements, and the app task drains both once a second. The phone builds everything else from the telemetry
(`web/src/shared/monitor`); see `docs/MONITORING.md`.

## Command authentication
The BLE link is not encrypted, so commands are protected at the application level. `features/auth` (`AuthGate`)
runs an HMAC-SHA256 challenge-response with a shared 32-byte key. The key is either built in (`FW_KEY=file`, `make fw-key`) or provisioned by the first client (`Provision`, answered to `Unclaimed`) and kept in NVS; `make fw-unclaim` erases it.
`App` drops every command and withholds status/scan telemetry until the session is authenticated; the BLE adapter
sends `Event::LinkReset` on every connect and disconnect so a session never outlives its connection. A claimed device accepts only its own key; an unclaimed one lets the first client claim it, so claim it right after flashing.

## Signed firmware updates
`features/ota` verifies an Ed25519 signature over the image SHA-256 (`OtaEndSigned`) before the writer may finish; any
failure aborts the update. Public key: `BEEWAVE_OTA_PUBKEY` at build time; private key: untracked `firmware/.beewave-ota.pem`
(`make fw-ota-key`). Keys travel into the core as `DeviceKeys`; a missing key fails closed.

## Diagnostics and tuning
Wi-Fi sniffing and BLE share one radio. Telemetry is stamped when queued (`runtime/timed.rs`); with `FEATURES=coex_debug`
the BLE task logs queue-wait and notify-call percentiles (`LatencyHistogram` in `crates/core/src/features/latency`).
Tuning knobs are named constants: `DWELL` and `HOP_ORDER` (`wifi_radio/hopper.rs`), the connection parameters
(`ble_gatt/tuning.rs`, off by default) and the queue depths (`runtime/channels.rs`). Procedure: `docs/TESTING.md`.

## Web
```
web/src/
  shared/      protocol/ (codec), transport/ (Transport port: BleTransport, MockTransport), auth/ (key + handshake), app/ (context + helpers), targets/
  features/    connection, auth, status, control, scanner, log, ...: each exports mount(root, ctx)
  main.ts      composition root
```
Features depend on `shared/` only, never on each other. UI state flows from `ctx.onTelemetry(kind, cb)`.
Develop without hardware: `make web-mock`.

## Wire protocol
`[ver u8][seq u8][postcard body]`, one packet per BLE write/notify, MTU >= 64. Rust is the source of truth;
`make vectors` emits golden vectors that `web` tests decode/encode byte-for-byte.
