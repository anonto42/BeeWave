# Tasks

## 1. Scanner Filtering Logic & Templates

- [x] 1.1 Create `web/src/features/scanner/filter.ts` implementing `filterNetworks` with RSSI and station filtering logic, verified with unit tests in `web/test/features/ui-logic.test.ts`
- [x] 1.2 Update `web/src/features/scanner/template.ts` with `<select data-filter>` toolbar options and station count badge in `rowHtml`, verified with template unit tests
- [x] 1.3 Update `web/src/features/scanner/mount.ts` to wire filtering, query station counts from `AssocStore`, and subscribe to `ctx.monitor.assoc.onChange(render)`

## 2. Integration, Documentation & Verification

- [x] 2.1 Update `docs/MONITORING.md` documenting Scanner proximity and client filtering
- [x] 2.2 Verify with `pnpm --dir web test` and `pnpm --dir web build`
- [x] 2.3 Run full workspace checks (`cargo fmt`, `cargo check -p beewave-core --no-default-features`, `cargo test`) and sync OpenSpec specs
