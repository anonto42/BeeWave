# Tasks

## 1. Geolocation & WiGLE Formatter
- [ ] 1.1 Implement `GpsTracker` in `web/src/shared/gps/tracker.ts` with coordinates tracking and test injection
- [ ] 1.2 Implement `web/src/shared/monitor/wigle.ts` with capability string mapping, channel frequency calculation, and CSV generation
- [ ] 1.3 Add unit tests in `web/test/shared/monitor/wigle.test.ts` for WiGLE formatting, frequency mapping, and GPS coordinate output

## 2. Store & App Context Integration
- [ ] 2.1 Extend `NetworkRecord` and `ScanEvent` in `web/src/shared/monitor/types.ts` with optional GPS coordinates
- [ ] 2.2 Wire `GpsTracker` into `ScanStore::ingest` so active coordinates stamp recorded networks
- [ ] 2.3 Expose `gps` tracker on `AppContext`

## 3. UI Integration
- [ ] 3.1 Add GPS toggle and WIGLE CSV export button in `web/src/features/scanner/template.ts` and `mount.ts`
- [ ] 3.2 Add WiGLE export option in `web/src/features/settings/import-export/mount.ts`

## 4. Verification & Documentation
- [ ] 4.1 Document wardriving and WiGLE export in `docs/MONITORING.md`
- [ ] 4.2 Verify entire project with `pnpm test` and `pnpm run build`
