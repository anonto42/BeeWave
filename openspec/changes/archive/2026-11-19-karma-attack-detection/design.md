# Design

## Context

See proposal.md - Why.
`AnomalyDetector` tracks up to 32 Access Points (`Entry`) in a bounded `Vec` in `crates/core/src/features/anomaly/`. Each entry already stores `ssid: u32` as the FNV-1a hash of the observed SSID (or 0 if hidden/empty).
When a beacon is observed for an existing entry:
`crates/core` currently updates `e.ssid = ssid;` without inspecting whether the previous `e.ssid` was non-zero and differed from the new `ssid`.

## Goals / Non-Goals

**Goals:**
- Detect when a known BSSID with an existing non-empty SSID (`e.ssid != 0`) advertises a new non-empty SSID (`ssid != 0` and `e.ssid != ssid`).
- Emit `AnomalyKind::KarmaSsidChange` with `detail: 0`.
- Update `KINDS = 6` in `crates/core/src/features/anomaly/types.rs`.
- Wire `karmaSsidChange` in web client protocol decoder, alerts mapping (severity `alert`), and mock world.

**Non-Goals:**
- Do not alert when an AP transitions to or from hidden (SSID hash 0).
- Do not alert during the initial learning window (`LEARN_TICKS = 30`).

## Decisions

1. **Detection on Existing Entry Observation**:
   - In `AnomalyDetector::observe`:
     ```rust
     if e.ssid != 0 && ssid != 0 && e.ssid != ssid {
         found = Some((AnomalyKind::KarmaSsidChange, 0));
     }
     ```
   - This fires before updating `e.ssid = ssid`.
   - Cooldown `COOLDOWN_TICKS` (30 s) is respected per kind.

2. **Zero Additional Field Storage**:
   - Reuses existing `e.ssid: u32`. Increasing `KINDS` from 5 to 6 expands `quiet_until` by one `u32` per entry (128 bytes total across the 32-entry table).

## Risks / Trade-offs

- [Router SSID reconfiguration in production]: Router reconfigurations are rare, and in security contexts, an AP changing its SSID on the air is a valid warning event that operators want to see.
