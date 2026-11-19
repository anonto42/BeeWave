# Proposal

## Why

In Wi-Fi security auditing and penetration testing, 4-way WPA/WPA2 handshakes and PMKIDs are captured to verify network password strength via offline dictionary attacks. The industry standard tool for this is Hashcat using Hash Mode 22000 (`WPA-PBKDF2-PMKID+EAPOL`). While BeeWave currently exports raw `.cap`/`.pcap` files for 4-way handshakes and `.22000` text files for PMKIDs (`WPA*01*`), extracting Hashcat-compatible format from 4-way handshakes previously required external CLI utilities (`hcxtools` / `hcxpcapngtool`).

Generating standard Hashcat Mode 22000 EAPOL hash lines (`WPA*02*...`) directly in the client and providing a 1-click "COPY 22000" clipboard button eliminates friction, allowing security auditors to immediately paste the hash string into Hashcat or password auditing pipelines without needing intermediate tools or downloading `.pcap` files.

## What Changes

- Implement `handshakeToHashcat22000(pair: HandshakePair, essid: string): string | null` in `web/src/shared/monitor/handshake-store.ts` to synthesize standard Hashcat mode 22000 `WPA*02*<MIC>*<AP_MAC>*<STA_MAC>*<ESSID_HEX>*<ANONCE>*<EAPOL_ZEROED_MIC>*<MESSAGE_PAIR>` hashes from captured EAPOL Message 1 (or 3) and Message 2 frames.
- Add `.22000` export and 1-click "COPY 22000" button in `web/src/features/handshake/` for completed 4-way handshakes alongside existing `.cap`/`.pcap` exports.
- Add 1-click "COPY" button for PMKID `.22000` rows with momentary visual feedback ("COPIED!").
- Add unit tests verifying `handshakeToHashcat22000` against standard Hashcat test vectors, byte offsets, and error handling for missing/truncated messages.

## Capabilities

### Modified Capabilities

- `web-client`: Add requirement for 4-way handshake Hashcat Mode 22000 export (`WPA*02*...`) and 1-click clipboard copy.

## Impact

- Affected files:
  - `web/src/shared/monitor/handshake-store.ts` (new function `handshakeToHashcat22000`, exported in `index.ts`)
  - `web/src/features/handshake/template.ts` (export & copy buttons for handshake and PMKID rows)
  - `web/src/features/handshake/mount.ts` (click handlers for `.22000` export and clipboard copy with visual feedback)
  - `web/test/shared/monitor/handshake-capture.test.ts` (unit tests for `handshakeToHashcat22000`)
- No wire protocol changes (`crates/protocol` remains untouched).
- No firmware changes (`crates/core` and `firmware/` remain untouched).
