# Design: Hidden SSID Automatic Decloaking

## Context & Motivation
Access points configured to hide their SSID broadcast IEEE 802.11 Beacon frames with an Information Element 0 (SSID) length of 0 or filled with zeroes (`\0\0\0...`). However, the 802.11 specification mandates that when a client attempts to connect to or probe for a hidden AP using a directed Probe Request, the AP must respond with a directed Probe Response frame containing the actual cleartext SSID.

Because BeeWave operates a promiscuous channel-hopping 802.11 receiver, `parse_beacon` receives both Beacon frames (subtype 0x80) and Probe Response frames (subtype 0x50).

In the current implementation:
1. `ScanTable::seen` stores `FnvIndexMap<[u8; 6], (i8, u32), TABLE_CAP>`: only `(rssi, reported_at)`.
2. When a hidden AP is first heard, it is stored with `rssi` and `reported_at`.
3. When a Probe Response containing the real SSID arrives shortly afterwards, the RSSI is typically within ±3 dB and `min_interval_ms` may not have elapsed. `ScanTable` evaluates `report = false` and drops the frame.
4. Even when reported, `ScanResult` carries no indication that the network was previously hidden and has now been uncloaked.

## Core Design

### 1. `ScanTable` Entry State
Replace the tuple `(i8, u32)` with an explicit entry struct:
```rust
#[derive(Debug, Clone, Copy)]
struct TableEntry {
    rssi: i8,
    reported_at: u32,
    had_empty_ssid: bool,
}
```

In `ScanTable::observe(o: Observation, now_ms: u32) -> Option<TelemetryMessage>`:
1. Check filter pass criteria. If `o.ssid` is empty and an SSID filter is active, it passes if the filter is empty.
2. Lookup `self.seen.get(&o.bssid)`.
3. If not seen before:
   - Insert new `TableEntry { rssi: o.rssi, reported_at: now_ms, had_empty_ssid: o.ssid.is_empty() }`.
   - Return `Some(TelemetryMessage::ScanResult { bssid: o.bssid, channel: o.channel, rssi: o.rssi, ssid: o.ssid })`.
4. If seen before with `prev`:
   - If `prev.had_empty_ssid && !o.ssid.is_empty()`:
     - **Decloak event detected!**
     - Update entry: `prev.had_empty_ssid = false; prev.rssi = o.rssi; prev.reported_at = now_ms;`
     - Return `Some(TelemetryMessage::DecloakedResult { bssid: o.bssid, ssid: o.ssid, channel: o.channel, rssi: o.rssi })`.
   - Otherwise (normal re-observation):
     - If `o.ssid.is_empty() && !prev.had_empty_ssid`:
       - Do not overwrite `prev.had_empty_ssid`.
     - Evaluate standard dedup:
       `now_ms.wrapping_sub(prev.reported_at) >= self.min_interval_ms && (o.rssi - prev.rssi).abs() >= self.dedup_db`.
     - If passing, update `(prev.rssi, prev.reported_at)` and return `Some(TelemetryMessage::ScanResult)`.

This maintains `#![no_std]` compliance and zero allocations (`FnvIndexMap` over fixed capacity).

## Wire Protocol
Add variant to `TelemetryMessage`:
```rust
DecloakedResult {
    bssid: [u8; 6],
    ssid: heapless::String<32>,
    channel: u8,
    rssi: i8,
}
```

## Web Client Design
1. In `web/src/shared/protocol/`: decode `DecloakedResult` (telemetry type tag 30).
2. In `web/src/shared/monitor/scan-store.ts`:
   - Extend `NetworkRecord` with `decloaked?: boolean`.
   - Add `ingestDecloaked(m: { bssid: string, ssid: string, channel: number, rssi: number })`.
   - If record exists, update `rec.ssid = m.ssid`, `rec.channel = m.channel`, `rec.rssi = m.rssi`, `rec.decloaked = true`.
3. In `web/src/shared/monitor/alerts.ts` / `alert-store.ts`:
   - Add handler for `decloaked` telemetry.
   - Format: `[DECLOAKED] Hidden network ${bssid} revealed as "${ssid}" on CH ${channel}`.
   - Severity: `warn` (or `info`).
4. In `web/src/features/scanner/template.ts`:
   - When `rec?.decloaked` is true, render `<span class="badge small warn">DECLOAKED</span>`.
   - Display the uncloaked SSID clearly in the scanner row.
