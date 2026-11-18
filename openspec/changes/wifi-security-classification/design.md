# Design

## Context
In 802.11 networks, management frames (Beacons and Probe Responses) advertise the authentication suites, cipher suites, and management frame protection (PMF) configuration of an Access Point. Currently, BeeWave extracts only SSID, channel, and WPS metadata from beacons. Without security classification, all discovered networks appear identical with respect to their encryption posture, and defenders cannot assess susceptibility to deauthentication or handshake interception.

## Goals / Non-Goals

**Goals:**
- Zero-allocation extraction of 802.11 Capability Information flags, RSN elements (Tag 48), and WPA1 elements (Tag 221 OUI `00:50:f2:01`).
- Accurate classification of modern Wi-Fi security protocols: `Open`, `Wep`, `Wpa`, `Wpa2`, `Wpa3`, `Wpa2Wpa3` (transition mode), `Owe` (Enhanced Open), and `Enterprise` (802.1X).
- Detection of Protected Management Frames (PMF / 802.11w) state: `None`, `Capable` (MFPC=1, MFPR=0), and `Required` (MFPR=1).
- Bounded, deduplicated emission via `SecurityTracker` (32 entries) in `crates/core` so security metadata is only sent when new or when security configuration changes.
- UI presentation of clear security badges and PMF tags in the Web Bluetooth PWA.

**Non-Goals:**
- Active beacon or probe injection (BeeWave remains strictly receive-only).
- Transmitting fake beacons or attempting PMF downgrade injections.
- Decrypting WEP/WPA payload traffic.

## Decisions

### Decision 1: Protocol placement and wire representation
We define `WifiSecurityProtocol` and `WifiPmfMode` unit enums in `crates/protocol/src/types.rs` and append `SecurityResult` as a variant of `TelemetryMessage`:
```rust
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiSecurityProtocol {
    Open,
    Wep,
    Wpa,
    Wpa2,
    Wpa3,
    Wpa2Wpa3,
    Owe,
    Enterprise,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiPmfMode {
    None,
    Capable,
    Required,
}
```
**Rationale**: Postcard serializes unit enums as single-byte varints. Appending `SecurityResult { bssid: [u8; 6], protocol: WifiSecurityProtocol, pmf: WifiPmfMode }` maintains backwards compatibility, avoids bloating routine beacon `ScanResult` packets, and keeps wire serialization minimal (8 bytes).

### Decision 2: Zero-allocation parsing in `crates/core/src/features/scan/security.rs`
The parser inspects:
1. `capabilities`: 2 bytes at offset 34 of beacon frame. Bit 4 (`0x0010`) is the Privacy flag.
2. Information Elements:
   - Tag 48 (RSN / 802.11i): Parses Group Cipher, Pairwise Ciphers, AKM suites, and RSN capabilities.
     - AKM `00-0F-AC:1` or `00-0F-AC:3` = Enterprise (802.1X / FT-802.1X)
     - AKM `00-0F-AC:2` or `00-0F-AC:4` or `00-0F-AC:6` = WPA2-PSK
     - AKM `00-0F-AC:8` or `00-0F-AC:9` = WPA3-SAE
     - AKM `00-0F-AC:18` = OWE (Enhanced Open)
     - If both PSK and SAE are present: `Wpa2Wpa3` Transition mode.
     - RSN Capabilities field (2 bytes LE):
       - Bit 6: MFPC (Management Frame Protection Capable)
       - Bit 7: MFPR (Management Frame Protection Required)
   - Tag 221 (Vendor Specific with OUI `00:50:f2` and Type `1`): WPA1.
   - If Privacy bit is 0 and no RSN/WPA element: `Open`.
   - If Privacy bit is 1 and no RSN/WPA element: `Wep`.

**Rationale**: Pure function over byte slices without heap allocations (`#![no_std]`). Bounds checks ensure robustness against malformed or truncated elements.

### Decision 3: Deduplication in `SecurityTracker`
Like `WpsTracker`, `SecurityTracker` stores up to 32 entries mapping `bssid -> (protocol, pmf)`. An incoming beacon only triggers a `SecurityResult` telemetry dispatch if the BSSID is unseen or if `(protocol, pmf)` differs from the cached state.
**Rationale**: Access point security parameters do not fluctuate on normal frames. Deduplication prevents filling the BLE sink buffer with redundant state.

## Risks / Trade-offs

- **[Malformed or non-standard beacon elements]** → All slice operations are strictly checked with lengths before reading; truncated or non-standard AKM suites gracefully fall back to safe defaults (e.g., standard WPA2 or Open if privacy bit is clear).
- **[Transition modes and complex multi-AKM beacons]** → If an AP advertises both PSK and SAE AKMs, it is explicitly classified as `Wpa2Wpa3` transition mode, which informs security analysts that legacy WPA2 downgrade attacks may be applicable.
