# Tasks

## 1. Monitor Kernel & Protocol Export

- [x] 1.1 Implement `handshakeToHashcat22000` in `web/src/shared/monitor/handshake-store.ts` and export it in `web/src/shared/monitor/index.ts`. Verify with unit tests in `web/test/shared/monitor/handshake-capture.test.ts`.
- [x] 1.2 Add test cases in `web/test/shared/monitor/handshake-capture.test.ts` covering M1+M2 (message_pair "00"), M3+M2 (message_pair "02"), missing M1, missing M2, short frame (<97 bytes), and verify against Hashcat Mode 22000 format.

## 2. Web UI Handshake Feature

- [x] 2.1 Update `web/src/features/handshake/template.ts` with `.22000` download and `COPY` buttons for completed 4-way handshakes and `COPY` button for PMKIDs.
- [x] 2.2 Update `web/src/features/handshake/mount.ts` with click handlers for `.22000` download and clipboard copy with momentary "COPIED!" visual feedback.
- [x] 2.3 Verify web tests and build with `pnpm test` and `pnpm run build`.

## 3. Verification & Specs

- [x] 3.1 Run `cargo test`, `pnpm test`, and `make spec` to ensure all tests pass cleanly.
