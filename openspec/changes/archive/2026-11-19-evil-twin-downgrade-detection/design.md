# Design

## Context
See `proposal.md` for motivation. Currently, `crates/core/src/features/anomaly/detector.rs` maintains a bounded table of `Entry` records (capacity 64) tracking BSSID, SSID hash, channel, cycle, and RSSI. Each `Observation` passed from the sniffer includes `pub security: Option<SecurityInfo>`, where `SecurityInfo` carries `protocol: WifiSecurityProtocol`.

## Goals / Non-Goals

**Goals:**
- Identify when an SSID known to use robust encryption (WPA2, WPA3, WPA2/WPA3, Enterprise) appears with Open or WEP security.
- Store security protocol information in `Entry` without heap allocations.
- Surface Evil Twin phishing threats with critical red alert severity in the PWA.

**Non-Goals:**
- Blocking or jamming the rogue AP (receive-only).
- Active authentication or deauthentication against the rogue AP.

## Decisions

### Decision 1: Protocol tracking in `Entry`
- *Choice*: Store `security_proto: Option<WifiSecurityProtocol>` in `Entry`.
- *Rationale*: When a new observation arrives with SSID hash matching an existing entry whose protocol was secure (`Wpa2`, `Wpa3`, `Wpa2Wpa3`, `Enterprise`), if the new observation is `Open` or `Wep`, flag as `AnomalyKind::SecurityDowngrade`.
- *RAM impact*: 1 byte per entry $\times$ 64 entries = 64 bytes in `AnomalyDetector`.

### Decision 2: Prioritizing Evil Twin over generic NewBssidForSsid
- *Choice*: If a new BSSID triggers both `SecurityDowngrade` and `NewBssidForSsid`, emit `SecurityDowngrade`.
- *Rationale*: A security downgrade is a high-severity active attack, whereas `NewBssidForSsid` is an informational warning.

## Risks / Trade-offs

- [Risk: Golden vectors mismatch] $\to$ Regenerate golden test vectors via `cargo test -p beewave-protocol --features std --test vectors` and verify TypeScript codec.
