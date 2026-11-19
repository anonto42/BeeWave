# Design

## Context

See proposal.md - Why.
BeeWave currently tracks Beacon floods (`BeaconFloodDetector`) and BLE advertising floods (`BleFloodDetector`). Probe requests (`0x40`) are parsed and deduplicated in `ProbeTracker` (32 entries), but high-rate probe request flood attacks (`mdk4 p`, `mdk3 p`, Marauder probe spam) with spoofed MACs easily bypass deduplication, exhaust the tracker, and flood the telemetry sink.

## Goals / Non-Goals

**Goals:**
- Implement `ProbeFloodDetector` in `crates/core/src/features/probe/flood.rs` without heap allocation in `#![no_std]`.
- Track probe frame rate and unique client MACs in a 1-second sliding window.
- When `rate >= 30` and `unique_clients >= 8`, emit `TelemetryMessage::ProbeFloodAlert { rate, unique_clients }`.
- Rate-limit alert emission to at most one per 5 seconds (`FLOOD_COOLDOWN_TICKS = 5`).
- Wire into `Monitoring` service and forward to the BLE sink.
- Mirror in `web/` protocol decoder, alerts formatter, and mock world simulator.

**Non-Goals:**
- No transmission of probe responses or active mitigation (strictly receive-only).

## Decisions

1. **Velocity + Diversity Heuristic**:
   - Similar to `BeaconFloodDetector`, we require both high packet rate (`rate >= 30`) and client address diversity (`unique_clients >= 8`). A single legitimate client aggressively scanning with high velocity will not trigger a flood alert; an attack tool spoofing random MACs will immediately trigger it.

2. **Zero Heap Allocation Bounded Vector**:
   - `unique_clients: Vec<[u8; 6], 32>`: Capped at 32 client samples per second. Total RAM cost is ~210 bytes.

3. **Alert Cooldown**:
   - A 5-second cooldown is enforced on the device core to avoid spamming the BLE control link when a flood persists.

## Risks / Trade-offs

- [Single client burst]: Mitigated by requiring `unique_clients >= 8` to distinguish attacker MAC spoofing from legitimate client probe bursts.
