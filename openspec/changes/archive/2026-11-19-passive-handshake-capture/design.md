## Parsing (pure, in `crates/core`, mirrors `scan/frame.rs::parse_beacon`)

`parse_eapol(frame: &[u8]) -> Option<EapolFrame>`:
1. Reject frames shorter than a minimal data header, frames with the Order bit set (HT Control field, not
   handled), and WDS frames (ToDS and FromDS both set).
2. Confirm the LLC/SNAP header (`AA AA 03 00 00 00`) and EtherType `0x888E` right after the (QoS-aware) header.
3. Confirm the EAPOL header's type byte is 3 (Key) and the body is at least the fixed 95-byte EAPOL-Key layout.
4. Read key info, replay counter, nonce and MIC at their fixed offsets.
5. Classify the message number from the ACK/MIC/Secure bits (message 1: ACK set, MIC clear; message 3: ACK and
   MIC set; message 2: MIC set, ACK and Secure clear; message 4: MIC and Secure set, ACK clear). Anything else
   (group-key frames, malformed) returns `None`.
6. Derive BSSID/client from ToDS/FromDS: `FromDS` -> BSSID is addr2, client is addr1; `ToDS` -> BSSID is addr1,
   client is addr2.
7. Copy at most 96 bytes of the EAPOL frame (from its own first byte) into a `heapless::Vec`, and set `truncated`
   when the real frame was longer.

## Tracking

`HandshakeTracker` (bounded, like `AnomalyDetector`): a small table keyed by (BSSID, client) holding a bitmask of
which messages have been seen. A pair is "complete" the moment it has message 2 (SNonce + MIC) and either
message 1 or message 3 (ANonce) — the minimum needed to attempt offline cracking. `HandshakeComplete` is emitted
once per pair.

## Wiring

`Monitoring` gains `handshake_enabled` (off by default, like the BLE monitor) and the tracker. `App::on_eapol_frame`
is a new driving port method, called from the app task exactly like `on_observation`. Firmware: the promiscuous
filter mask becomes `MGMT | DATA`; the callback tries `parse_eapol` on data frames the same way it already tries
`parse_beacon` on management frames, and only queues a result (never raw bytes) — no new allocation, no change to
the existing deauth-counting path.
