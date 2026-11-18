# Design

## Context

In WPA2/WPA3 networks supporting PMK caching (IEEE 802.11i), Access Points provide a PMKID to stations during EAPOL Message 1 inside the Key Data field. The sniffer promiscuous filter already captures EAPOL data frames. This design introduces zero-copy parsing of PMKID Key Data Encapsulation (KDE) blocks within EAPOL frames, a bounded deduplication table in `crates/core`, and wire/web support for display and Hashcat 22000 export.

## Goals / Non-Goals

**Goals:**
- Zero-copy, allocation-free extraction of PMKID from naturally occurring EAPOL Message 1 frames.
- Bounded memory footprint (heapless only) on device path.
- Clean integration with existing `features/handshake` module and `TelemetrySink`.
- Direct export in Hashcat mode 22000 format from the Web client.

**Non-Goals:**
- No active association or frame transmission (strictly passive).
- No on-device dictionary cracking.

## Decisions

### 1. PMKID Key Data Encapsulation (KDE) Parsing
In EAPOL-Key frames (Descriptor Type 2 or 254), bytes 97..99 encode `key_data_length` (u16 big-endian). The Key Data begins at byte offset 99 and contains Type-Length-Value (TLV) encapsulations.
A PMKID KDE follows IEEE 802.11i Table 20da:
- Tag: `0xDD` (Vendor Specific)
- Length: `0x14` (20 bytes: 4 bytes OUI/DataType + 16 bytes PMKID)
- OUI: `0x00, 0x0F, 0xAC` (IEEE 802.11)
- Data Type: `0x04` (PMKID KDE)
- Value: `[u8; 16]` PMKID

The parser `extract_pmkid(eapol: &[u8]) -> Option<[u8; 16]>` advances through Key Data TLVs using safe slice indexing (`.get()`), without allocation or panics.

### 2. Core Deduplication & Storage Budget
To avoid flooding the BLE telemetry stream on repeated beacons or reconnect attempts, `HandshakeTracker` incorporates a bounded deduplication buffer:
- `pmkids: heapless::Vec<([u8; 6], [u8; 16]), 16>`
- When full, the oldest entry is evicted.
- **RAM cost**: 16 * 22 bytes = ~352 bytes in RAM.
- **Flash cost**: ~1.5 KB Xtensa code addition.

### 3. Wire Protocol & Web Format
- Telemetry: `TelemetryMessage::PmkidFound { bssid: [u8; 6], client: [u8; 6], pmkid: [u8; 16] }`
- Hashcat 22000 format output:
  `WPA*01*<pmkid_hex>*<bssid_hex>*<client_hex>*<essid_hex>***`

## Failure Modes & Edge Cases
- **Truncated Key Data**: If `key_data_length` specifies bytes beyond the frame buffer, parser returns `None`.
- **Malformed TLVs**: If TLV length exceeds remaining buffer or causes overflow, iteration terminates immediately.
- **WPA3 SAE / PMKID Caching**: Extracted identically; if present in Key Data, PMKID is preserved.
