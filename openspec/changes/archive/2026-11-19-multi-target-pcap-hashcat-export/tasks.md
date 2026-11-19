# Tasks

## 1. Monitor PCAP & Hashcat Core Utilities

- [x] 1.1 Implement `buildMultiPcap` in `web/src/shared/monitor/pcap.ts` and verify with unit tests in `web/test/shared/monitor/pcap.test.ts`
- [x] 1.2 Implement `allPmkids` and `exportAll22000` in `web/src/shared/monitor/handshake-store.ts` and verify with unit tests in `web/test/shared/monitor/handshake-capture.test.ts`
- [x] 1.3 Export new utilities from `web/src/shared/monitor/index.ts`

## 2. Web UI & Target Workspace Integration

- [x] 2.1 Update `handshakeHtml` in `web/src/features/handshake/template.ts` with batch export toolbar controls
- [x] 2.2 Wire batch download and clipboard copy handlers in `web/src/features/handshake/mount.ts`
- [x] 2.3 Verify responsive layout and button states in `web/test/features/monitor-panels.test.ts`

## 3. Verification & Documentation

- [x] 3.1 Document multi-target PCAP and Hashcat 22000 export in `docs/MONITORING.md`
- [x] 3.2 Run vitest suite and pnpm build to confirm all tests pass
