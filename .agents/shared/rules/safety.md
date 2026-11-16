# Safety & Security Rules

1. No secrets, keys or bonding data in source control.
2. Validate every `DeviceCommand` in the core (ranges, state, sizes) before touching hardware.
3. OTA: strictly sequential offsets, size-bounded, SHA-256 verified before `set_boot_partition`, rollback enabled, mark-valid only after self-test.
4. BLE command characteristic requires encryption + authentication; never expose reboot/OTA on an open link.
5. The sniffer is for networks and devices the user owns or is authorized to test.
6. Never flash without confirming the chip (`make board-info`) and the target port.
