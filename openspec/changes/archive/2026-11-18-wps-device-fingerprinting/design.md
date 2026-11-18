# Design

## Context
See `proposal.md` for motivation. 802.11 Beacons and Probe Responses contain vendor-specific Information Elements (`id 221 / 0xDD`) with OUI `00:50:f2:04` holding Wi-Fi Protected Setup (WPS) attributes.

## Goals / Non-Goals

**Goals:**
- Zero-allocation TLV parser in `#![no_std]` for WPS vendor elements.
- Extract WPS Version, State, AP Setup Lock, Selected Registrar, Device Name, Manufacturer, and Model Name.
- Bounded deduplication on device to prevent redundant transmissions of static device metadata.
- Clear visual badges and hardware device details in the Web UI.

**Non-Goals:**
- Active WPS PIN brute-forcing (e.g. Reaver/Bully attacks); BeeWave is strictly passive and receive-only.
- Modifying AP configuration via WPS.

## Decisions

### 1. Zero-Allocation TLV Parsing
*Decision*: Parse the WPS payload linearly using big-endian 16-bit attributes.
- Buffer safety: Check that `i + 4 <= body.len()` and `i + 4 + attr_len <= body.len()`.
- String sanitization: Non-UTF-8 or invalid bytes are cleaned, and lengths are truncated to 32 bytes (`heapless::String<32>`).
*Rationale*: Ensures resilience against malformed or malicious beacons while keeping heap allocations at zero.

### 2. Bounded Deduplication
*Decision*: Track reported WPS status in a 32-entry bounded table (`WpsTracker`).
- Each entry stores `(bssid: [u8; 6], state: u8, locked: bool, selected_registrar: bool)`.
- If an AP's BSSID is already known and its lockout and registrar state haven't changed, subsequent beacons suppress `WpsResult` emission.
*Rationale*: Static device names and models need to be transmitted only once per scan session, conserving BLE airtime.

## Risks / Trade-offs

- [Risk] Some APs broadcast oversized or fragmented vendor elements.
  → *Mitigation*: The parser checks bounds strictly before every slice access; truncated elements yield partial records without panics.
- [Risk] Beacons without WPS elements.
  → *Mitigation*: Parsing returns `Option::None`, gracefully ignoring non-WPS networks without overhead.
