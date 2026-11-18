# Tasks

## 1. Protocol
- [x] 1.1 Append `PmkidFound { bssid: [u8; 6], client: [u8; 6], pmkid: [u8; 16] }` to `TelemetryMessage` in `crates/protocol/src/types.rs`
- [x] 1.2 Update golden wire vectors (`make vectors`) and mirror codec in `web/src/shared/protocol/`

## 2. Core Logic & Unit Tests
- [x] 2.1 Implement `extract_pmkid` in `crates/core/src/features/handshake/` with zero allocation and safe slice indexing
- [x] 2.2 Add host unit tests in `crates/core/src/features/handshake/tests.rs` for valid PMKID KDE, truncated key data, and malformed TLVs
- [x] 2.3 Add PMKID deduplication in `HandshakeTracker` and route `PmkidFound` publication in `App::on_eapol_frame`

## 3. Web Client
- [x] 3.1 Add PMKID tracking to `web/src/shared/monitor/handshake-store.ts`
- [x] 3.2 Render PMKID under target networks in `web/src/features/handshake/`
- [x] 3.3 Implement Hashcat mode 22000 export formatter (`WPA*01*...`)

## 4. Verification & Documentation
- [x] 4.1 Update `docs/MONITORING.md` with passive PMKID capture documentation
- [x] 4.2 Verify entire project with `make check`
