# Design: Real-Time BLE Flood and Spam Attack Detection

## Context
BLE advertisement spam attacks transmit high volumes of spoofed advertising packets to disrupt nearby consumer electronics. Attack tools specifically target:
- Apple ecosystem: Proximity pairing popups, AirDrop requests, and setup modals (`AdClass::Proximity`).
- Android ecosystem: Google Fast Pair device prompts (`AdClass::FastPair`).
- Windows ecosystem: SwiftPair prompts and generic device beacons (`AdClass::Generic`).

In a live attack, packet rates spike from ambient baseline levels (1–8 packets/second) to 30–80+ packets/second. The existing `BleMonitor` calculates rolling averages over a 60-second window with a 30-second warmup period, making it unsuitable for detecting rapid transient or burst-mode spam attacks.

## Core Design

### 1. Zero-Allocation `BleFloodDetector`
Located at `crates/core/src/features/ble_monitor/flood.rs`:
```rust
pub const DEFAULT_BLE_FLOOD_THRESHOLD: u16 = 30; // 30 advertisements per second
pub const BLE_FLOOD_COOLDOWN_TICKS: u8 = 5;

pub struct BleFloodDetector {
    threshold: u16,
    counts: [u16; 4], // counts per AdClass in the current 1-second window
    cooldown: u8,
}
```

- **Arrival Tracking (`observe`)**:
  When a BLE advertisement is classified, increment `counts[class_index(class)]`.
- **Evaluation on 1 Hz Tick (`tick`)**:
  - Calculate total rate across all 4 classes: `rate = sum(counts)`.
  - Identify dominant class (class with maximum packets in the window).
  - Clear `counts` to zero for the next window.
  - If `cooldown > 0`, decrement cooldown and return `None`.
  - If `threshold > 0 && rate >= threshold && cooldown == 0`:
    - Set `cooldown = BLE_FLOOD_COOLDOWN_TICKS` (5 seconds).
    - Emit `Some(TelemetryMessage::BleFloodAlert { rate, class: dominant_class })`.

### 2. Integration into `Monitoring`
In `crates/core/src/app/monitoring.rs`:
- Hold `ble_flood: BleFloodDetector` inside `Monitoring`.
- In `on_ble_adv(class)`: when `self.cfg.ble_enabled`, call `self.ble_flood.observe(class)`.
- In `on_tick()`: when `self.cfg.ble_enabled`, check `self.ble_flood.tick()` and push `BleFloodAlert` to output telemetry messages.
- In `reset()`: call `self.ble_flood.reset()`.

## Wire Protocol
Add variant to `TelemetryMessage`:
```rust
BleFloodAlert {
    rate: u16,
    class: AdClass,
}
```

## Web Client Design
1. In `web/src/shared/protocol/`: Decode `bleFloodAlert` (telemetry type tag 29).
2. In `web/src/shared/monitor/types.ts`: Add `"bleFlood"` to `AlertSource`.
3. In `web/src/shared/monitor/alerts.ts`:
   - Map `bleFloodAlert` to `NewAlert`:
     - `source: "bleFlood"`
     - `severity: "alert"`
     - `title: "BLE advertisement flood: ${m.rate}/s"`
     - `detail: "${CLASS_LABEL[m.class]} burst (possible Flipper Zero / Hydra-ESP BLE spam)"`
4. In `web/src/shared/transport/mock-world.ts`:
   - Simulate a BLE flood attack at regular intervals.
