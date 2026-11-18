## 1. Protocol
- [x] 1.1 Append the two commands and three telemetry variants; golden vectors
## 2. Core
- [x] 2.1 `parse_eapol` with host tests for every message, truncation, and the rejects (WDS, Order bit, wrong ethertype, non-Key EAPOL, bad bit combination)
- [x] 2.2 `HandshakeTracker` with host tests for completion and eviction; wire into `Monitoring` and `App`
## 3. Firmware
- [x] 3.1 Add the DATA filter bit, a bounded queue, and the callback path; boot-check the board (no panic, heap flat)
## 4. Web
- [x] 4.1 Panel: captured pairs, messages, hex export; mock world produces a sample handshake; tests
## 5. Docs
- [x] 5.1 `docs/MONITORING.md` section, explicitly stating the rejected deauth approach and why
