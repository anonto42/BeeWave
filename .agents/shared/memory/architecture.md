# Architecture Decisions

- 2026-11-16: classic ESP32-WROOM chosen; esp-idf-svc (std) over esp-hal for coex/promiscuous/OTA-rollback maturity.
- NimBLE via `esp32-nimble`, BLE-only controller (classic BT memory released). Expect ~80-120 KB free heap, not 180 KB.
- postcard wire format with `[ver][seq]` header; no fragmentation, requires MTU >= 64. iOS Safari lacks Web Bluetooth.
- PWA is hosted over HTTPS (not on the ESP32); SPIFFS removed, 2x1.5 MB OTA slots.
- Hexagonal core in `crates/core` so all rules are host-testable; firmware is adapters + composition root only.
