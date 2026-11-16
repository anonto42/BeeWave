## 1. Protocol
- [x] 1.1 Append auth variants to `DeviceCommand` and `TelemetryMessage`; golden vectors
- [x] 1.2 Mirror in `web/src/shared/protocol`

## 2. Core
- [x] 2.1 `System::fill_random`, `features/auth` slice with host tests
- [x] 2.2 Gate `App` commands and telemetry; `App::on_link_reset`

## 3. Firmware
- [x] 3.1 Parse `BEEWAVE_KEY`, `Event::LinkReset`, `fill_random` adapter
- [x] 3.2 `make fw-key` and wire key into fw targets

## 4. Web
- [x] 4.1 Key store, HMAC, handshake with tests (including a Rust-generated MAC vector)
- [x] 4.2 Auth feature panel and auto-handshake on connect

## 5. Docs
- [x] 5.1 Update USAGE, TESTING, ARCHITECTURE and agent notes
