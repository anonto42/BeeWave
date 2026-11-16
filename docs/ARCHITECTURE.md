# Architecture

Hexagonal (ports & adapters) inside, feature slices on the outside. Same rules in Rust and TypeScript.

```
crates/protocol   wire types + postcard codec            (no_std, depends on nothing)
crates/core       domain + ports + app service           (no_std, depends on protocol only)
firmware/         ESP-IDF adapters + composition root    (depends on core; ONLY crate with esp-idf)
web/              PWA, feature slices                    (mirrors protocol.ts; Transport = the port)
```

## Dependency rule
`adapters -> core -> protocol`. Never the reverse. `core` has no `esp-idf`, no threads, no heap allocation
(heapless only) so every rule is unit-tested on the host with `make test`.

## Rust core
```
crates/core/src/
  ports.rs               traits the outside world must implement (TelemetrySink, RadioDriver, System, OtaWriter)
  app.rs                 App: routes driving input (commands, frames, ticks) to slices, publishes via the sink
  features/<slice>/      state + rules + tests for ONE capability: radio, scan, status, ota, settings, auth
```
Slices depend on `ports` and `protocol`, never on each other. `App` is the only place that composes them.

### Add a feature (Rust)
1. Wire message(s) in `crates/protocol` -> `make vectors` -> mirror in `web/src/shared/protocol.ts`.
2. `crates/core/src/features/<name>/mod.rs` with its state machine and tests; new port in `ports.rs` if it needs hardware.
3. Route it in `App::on_command` / `on_observation` / `on_tick`.
4. Implement the port in `firmware/src/adapters/<name>.rs`; construct it in `firmware/src/main.rs` (composition root).

## Firmware
`adapters/*` = every ESP-IDF call. `runtime.rs` = threads + bounded channels. `main.rs` = wiring only.
Driving adapters push `Event`s into one bounded channel; the `app` task owns `App` (no locks needed).
Telemetry is lossy by contract: `BleSink::publish` never blocks, drops are counted and reported in `Status.dropped`.

## Command authentication
The BLE link is not encrypted, so commands are protected at the application level. `features/auth` (`AuthGate`)
runs an HMAC-SHA256 challenge-response with a shared 32-byte key (`BEEWAVE_KEY` at build time, see `make fw-key`).
`App` drops every command and withholds status/scan telemetry until the session is authenticated; the BLE adapter
sends `Event::LinkReset` on every connect and disconnect so a session never outlives its connection. No key = locked.

## Web
```
web/src/
  shared/      protocol.ts (codec), transport.ts (Transport port: BleTransport, MockTransport), app.ts (context + helpers)
  features/    connection, auth, status, control, scanner, log, ...: each exports mount(root, ctx)
  main.ts      composition root
```
Features depend on `shared/` only, never on each other. UI state flows from `ctx.onTelemetry(kind, cb)`.
Develop without hardware: `make web-mock`.

## Wire protocol
`[ver u8][seq u8][postcard body]`, one packet per BLE write/notify, MTU >= 64. Rust is the source of truth;
`make vectors` emits golden vectors that `web` tests decode/encode byte-for-byte.
