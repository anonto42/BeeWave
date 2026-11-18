# Design

## Context
See `proposal.md` for motivation. Currently, BeeWave detects Access Points via Beacons and probing clients via Probe Requests. Passive monitoring of 802.11 Data frame headers provides complete association awareness (which clients are communicating with which BSSIDs).

## Goals / Non-Goals

**Goals:**
- Zero-copy, zero-allocation parsing of 802.11 Data/QoS Data frame headers (`0x08`, `0x88`, etc.).
- Robust direction detection via 802.11 `ToDS` / `FromDS` control bits.
- Bounded memory tracker (`heapless::Vec`) with LRU eviction and 6 dB / 5s deduplication on the ESP32.
- Clean presentation of connected stations under target APs in the Web Bluetooth UI with vendor OUI resolution.

**Non-Goals:**
- Decrypting data payloads or inspecting encapsulated IP/TCP/UDP packets (payloads are ignored).
- Active deauth or disassociation injection (BeeWave is strictly receive-only).
- Unlimited in-memory station history on device.

## Decisions

### 1. Data Frame Address Resolution via ToDS/FromDS
*Decision*: Inspect Frame Control byte 1 (`fc1`):
- `ToDS = 1, FromDS = 0` (Client -> AP): Client MAC is Address 2 (TA), AP BSSID is Address 1 (RA).
- `ToDS = 0, FromDS = 1` (AP -> Client): Client MAC is Address 1 (RA), AP BSSID is Address 2 (TA).
- `ToDS = FromDS` (0,0 for IBSS or 1,1 for WDS bridge): Ignored.
*Rationale*: Standard IEEE 802.11 addressing rules. Filtering multicast/broadcast (`mac[0] & 0x01 != 0`) and all-zero addresses ensures only valid unicast client associations are emitted.

### 2. Memory Footprint and Deduplication Budget
*Decision*: Fixed table of 32 association pairs (`ASSOC_CAPACITY = 32`) in `crates/core`.
- Per-entry size: 24 bytes (6 bytes BSSID + 6 bytes Client + 1 byte channel + 1 byte RSSI + 4 bytes tick + 4 bytes counter + padding).
- Total RAM cost: ~768 bytes in core RAM.
- Deduplication: Emits only when a pair is first seen, or when RSSI moves by >= 6 dB, or when >= 5 seconds pass.

### 3. Queue Separation in Firmware
*Decision*: Introduce `ASSOC_DEPTH = 32` bounded `SyncSender`/`Receiver` in `channels.rs`.
*Rationale*: Separates high-frequency data frame headers from EAPOL handshake frames and beacon observations, preventing data traffic bursts from starving handshakes or commands.

## Risks / Trade-offs

- [Risk] High data packet volume could overwhelm the app task queue.
  → *Mitigation*: The sniffer callback drops on full queue (`try_send`), recording dropped counts in telemetry status without blocking or allocating.
- [Risk] Randomized MAC addresses on client devices.
  → *Mitigation*: The Web UI's `vendorOf` already detects Locally Administered Addresses (`mac[0] & 0x02 != 0`) and tags them as "Randomized / Private MAC".
