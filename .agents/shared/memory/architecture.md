# Architecture Decisions

- 2026-11-16: classic ESP32-WROOM chosen; esp-idf-svc (std) over esp-hal for coex/promiscuous/OTA-rollback maturity.
- NimBLE via `esp32-nimble`, BLE-only controller (classic BT memory released). Expect ~80-120 KB free heap, not 180 KB.
- postcard wire format with `[ver][seq]` header; no fragmentation, requires MTU >= 64. iOS Safari lacks Web Bluetooth.
- PWA is hosted over HTTPS (not on the ESP32); SPIFFS removed, 2x1.5 MB OTA slots.
- Hexagonal core in `crates/core` so all rules are host-testable; firmware is adapters + composition root only.
- 2026-11-18: Client association tracking via bounded LRU `AssocTracker` without allocating memory; passive handshake listener strictly non-transmitting.
- 2026-11-19: Multi-target PCAP ring buffer and 22000 Hashcat export; consolidated offline analysis in browser sandbox.
- 2026-11-19: Disassociation frame (`0x00A0`) sniffing and sliding-window rate evaluation; Apple Find My (`0x004C`, type `0x12`) BLE advertisement classification; probe request flood detection in core.
- 2026-11-19: Anomaly detection engine expanded with Evil Twin security downgrade detection and KARMA rogue AP multi-SSID mutation tracking.
- 2026-11-20: Wi-Fi PHY standard generation parsing in zero-copy beacon parser (Tag 45 HT, Tag 42 ERP, Tag 255 Ext 35 HE) with bounded 32-entry `PhyTracker`.
- 2026-11-20: Monotonic 64-bit microsecond TSF rollback detector in `AnomalyDetector` flagging rogue AP timestamp inversions $\ge 5$ s.
- 2026-11-20: Scanner proximity and client association filtering (`filterNetworks`) and live station badging in PWA.
- 2026-11-20: Mandatory agent knowledge synchronization rule (`.agents/shared/rules/agent-maintenance.md`) to maintain `.agents/` memory on all changes.
