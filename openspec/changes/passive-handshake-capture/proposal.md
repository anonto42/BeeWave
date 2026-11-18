## Why

A security-minded owner wants to know when a WPA/WPA2 four-way handshake happens near the device (their own
re-authenticating client, or evidence of one on a network they are auditing with permission) without BeeWave
ever transmitting anything. The existing sniffer already parses management frames (beacons); this extends it
to the EAPOL key frames carried in data frames, strictly for frames that occur naturally.

## What Changes

- **New receive-only core slice** `handshake`: parses the 802.11 data-frame header and EAPOL-Key body, classifies
  which of the four handshake messages it is from the key-info bits, and tracks per (BSSID, client) pair until
  a crackable pair (an ANonce-carrying message plus the SNonce+MIC message) has been seen.
- **Wire protocol (append-only, version stays 1):** `SetHandshakeCapture`, `GetHandshakeCapture` commands;
  `HandshakeCaptureState`, `HandshakeFrame`, `HandshakeComplete` telemetry.
- **Firmware:** the promiscuous filter also admits data frames; the callback forwards only ones that parse as
  EAPOL-Key (everything else is rejected in the first few bytes, cheaply) into a new bounded queue, parsed the
  same way beacons are (a pure core function, no allocation in the callback).
- **Web:** a panel listing captured pairs and their messages, with a hex export of what was captured per message.

## Explicitly rejected approach

The request that prompted this also described sending spoofed deauthentication frames to force a client to
reconnect on demand. That is an active disruption of a device's connection and is out of scope for BeeWave by
design (no transmit path besides advertising and GATT notifications, see invariant 4). This change captures
only handshakes that occur without any prompting from the device.

## Limitations

- No client connected nearby -> nothing to capture (this is inherent to being passive).
- 802.11w / WPA3 does not change what is captured here (that protection is against forged deauth, which this
  feature never sends); handshake frames themselves are not protected by MFP.
- Each `HandshakeFrame` carries the raw EAPOL bytes capped to 96 bytes (`MAX_BODY` budget) to fit one packet; a
  message that does not fit is flagged `truncated` and still classified, but may not be enough for an external
  cracking tool. This is a detection and logging feature, not a guaranteed capture pipeline.

## Impact

`crates/protocol`, new `crates/core/src/features/handshake/`, `app/monitoring.rs` (wires it in like the deauth
and BLE monitors), `firmware/adapters/wifi_radio` (filter mask, callback, a new queue), `web` (a panel + tests),
docs.
