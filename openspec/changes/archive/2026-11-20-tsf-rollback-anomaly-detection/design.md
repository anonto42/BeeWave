# Design: Beacon Timestamp (TSF) Rollback Detection

## Context

See `proposal.md` for background and motivation.
Beacon management frames carry an 8-byte (64-bit) microsecond hardware timestamp at byte offset 24..32 (`MGMT_HEADER_LEN..MGMT_HEADER_LEN + 8`). Genuine APs increment this timestamp continuously. Software rogue APs (`airbase-ng`, `wifiphisher`) reset or emit non-monotonic timestamps.

## Goals / Non-Goals

**Goals:**
- Extract `tsf: u64` in `parse_beacon` without memory allocation.
- Track `last_tsf: u64` per BSSID in `AnomalyDetector`'s bounded entry table (64 entries).
- Detect backward rollbacks $\ge 5,000,000\,\mu\text{s}$ (5 seconds).
- Rate limit alerts using existing 30-second cooldown per kind per BSSID.

**Non-Goals:**
- Detecting forward timestamp jumps (an authentic AP jumping forward in time can happen during clock sync / NTP adjustments; only backward jumps indicate fake AP injection or reboot).

## Decisions

### 1. Rollback Threshold
- **Decision**: 5,000,000 microseconds (5 seconds).
- **Rationale**: Filters out minor frame reordering or multipath jitter while reliably catching software rogue APs that start near 0.

### 2. Detail Payload
- **Decision**: Detail field holds the rollback magnitude in whole seconds: `(rollback_us / 1_000_000).min(255) as u8`, clamped to at least 1.
- **Rationale**: Fits in the standard 1-byte `detail: u8` field of `AnomalyAlert`.

## Risks / Trade-offs

- **[Risk] AP hardware reboot** $\to$ *Mitigation*: A legitimate AP rebooting will reset its uptime to 0, which also triggers this alert once; in a security monitoring context, knowing an AP rebooted or was power-cycled is valuable situational awareness.
