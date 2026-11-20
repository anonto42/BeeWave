# Technical Design: Scanner Station Badging & Proximity Filter

## Context
See proposal.md for motivation. `AssocStore` in `web/src/shared/monitor/assoc-store.ts` tracks client stations associated with BSSIDs from passive data frame interception (`assocResult`). The Scanner feature slice currently only provides sorting (`sortNetworks`) and renders access points without client awareness or proximity filtering.

## Goals / Non-Goals

**Goals:**
- Provide a pure filtering function in `web/src/features/scanner/filter.ts` that filters networks based on RSSI thresholds (`>= -80`, `>= -70`, `>= -60`) or associated client count (`> 0`).
- Update `web/src/features/scanner/template.ts` with a toolbar filter `<select data-filter>` and an AP card badge for connected stations.
- Wire `filterNetworks` into `web/src/features/scanner/mount.ts` and subscribe to `ctx.monitor.assoc.onChange(render)` to ensure reactive updates.
- Display filtered count vs total count in `[data-count]` (e.g. `4 / 12 networks`).

**Non-Goals:**
- Firmware or wire protocol alterations: relies strictly on existing `AssocResult` telemetry and `AssocTracker`.
- Modifying `AssocStore` storage invariants.

## Decisions

### 1. Dedicated `filter.ts` Module
In accordance with `.agents/shared/rules/file-structure.md` ("one concern per file"), filtering logic will live in `web/src/features/scanner/filter.ts` rather than bloating `sort.ts` or `mount.ts`.
- `filterNetworks(list, filterKey, stationCountFn)`
- Types: `FilterKey = "all" | "-80" | "-70" | "-60" | "stations"`

### 2. Badge Styling & Semantics
The badge will be rendered as:
`<span class="badge small ok" title="${count} connected client station${count === 1 ? '' : 's'}">${count} ${count === 1 ? 'CLIENT' : 'CLIENTS'}</span>`
Using `.badge.ok` from `web/src/style.css` provides clear visual recognition (green outline/fill) indicating an active target with connected clients suitable for handshake monitoring.

### 3. Reactive Subscription
In `web/src/features/scanner/mount.ts`, `ctx.monitor.assoc.onChange(render)` will be registered alongside `ctx.monitor.scans.onChange(render)` and `ctx.targets.onChange(render)`. This ensures that when an association telemetry packet arrives, any visible AP card instantly gains or updates its client badge.

## Risks / Trade-offs

- **[Filter state on scan clear]** → When the user clicks CLEAR, both the network map and filter display reset cleanly without desynchronizing.
- **[Performance on rapid packet ingestion]** → The Scanner rendering is already bounded by scan interval and DOM diffing; `AssocStore` deduplication (`DEDUP_INTERVAL_SECS` / RSSI delta) prevents excess event flooding.
