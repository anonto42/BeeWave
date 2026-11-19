# Design

## Context

See `proposal.md` for motivation.
Currently, BeeWave captures 4-way handshakes and PMKIDs via `crates/core/src/features/handshake/`. The Web client displays captured pairs in `web/src/features/handshake/` and offers `.cap`/`.pcap` downloads. PMKIDs have a `.22000` download button using `pmkidToHashcat22000`. Handshake hashes (`WPA*02*...`) are not yet formatted, requiring users to install and run `hcxpcapngtool`.

## Goals / Non-Goals

**Goals:**
- Provide client-side pure function `handshakeToHashcat22000` generating valid Hashcat Mode 22000 strings (`WPA*02*...`) from captured handshake pairs.
- Add `.22000` file download for completed handshakes.
- Add 1-click clipboard "COPY" buttons for both 4-way handshakes and PMKIDs with momentary UI feedback ("COPIED!").
- Maintain zero backend/firmware overhead and 100% offline client operation.

**Non-Goals:**
- No active frame injection or deauthentication frames.
- No client-side GPU/WPA dictionary cracking (cracking runs in Hashcat on the auditor's workstation).

## Decisions

### Decision 1: EAPOL-Key Frame Offset Extraction
In IEEE 802.11i / 802.1X:
- An EAPOL-Key frame starts with a 4-byte header (`version`, `type`, `length`).
- The fixed Key Descriptor section is 95 bytes (total 99 bytes):
  - `17..49`: Key Nonce (32 bytes). In M1/M3, this is the Authenticator Nonce (`ANonce`).
  - `81..97`: Key MIC (16 bytes). In M2, this contains the client Message Integrity Code.
- Hashcat Mode 22000 requires the EAPOL payload of Message 2 with its MIC bytes (81..97) zeroed out so Hashcat can recompute and compare the candidate MIC.
- `handshakeToHashcat22000` will:
  1. Retrieve M1 (or fallback to M3) for `ANonce`.
  2. Retrieve M2 for `MIC` and `EAPOL`.
  3. Validate buffer lengths (`m1.eapol.length >= 49`, `m2.eapol.length >= 97`).
  4. Clone M2's `eapol`, zero bytes 81..97, and convert to hex.
  5. Form line: `WPA*02*${mic}*${apMac}*${staMac}*${essidHex}*${anonce}*${eapolZeroed}*${messagePair}` (`00` for M1+M2, `02` for M3+M2).

### Decision 2: Clipboard Copy with Graceful Fallback
Use `navigator.clipboard.writeText(text)` when available, falling back to a temporary textarea selection with `document.execCommand("copy")` if unavailable in certain browser security contexts. Provide immediate visual feedback on the button (displaying "COPIED!" for 1500ms).

## Risks / Trade-offs

- **[Risk]** Incomplete or truncated EAPOL frames (due to device `MAX_EAPOL_CAPTURE = 100`).
  → **Mitigation:** The fixed Key Descriptor (up to byte 99) contains all header fields, Nonce, IV, and MIC. If `m2.eapol.length < 97`, `handshakeToHashcat22000` returns `null` safely, preventing malformed hash generation.
