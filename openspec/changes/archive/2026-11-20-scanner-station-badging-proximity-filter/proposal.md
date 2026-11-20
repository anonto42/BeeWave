# Proposal: Scanner Connected Station Badging & Proximity Filtering

## Why
In 802.11 reconnaissance, physical site surveys, and wardriving, operators face two primary friction points: (1) distant, low-signal APs clutter the scanner view, obscuring nearby targets; and (2) active targets with connected client stations cannot be identified without navigating away to the separate Client Stations tab or clicking into individual targets. Adding proximity RSSI filtering and live connected station badges directly to the Scanner network cards enables immediate discovery and targeting of active, nearby access points.

## What Changes
- Add a proximity and client filter selector to the Scanner toolbar (`ALL`, `> -80 dBm`, `> -70 dBm`, `> -60 dBm`, and `WITH CLIENTS`).
- Filter network lists according to the selected RSSI threshold or presence of associated client stations in `web/src/features/scanner/filter.ts`.
- Update network card count display to reflect both filtered and total network counts (e.g. `4 / 12 networks`).
- Render a color-coded connected station count badge (e.g. `1 CLIENT` or `N CLIENTS`) on Access Point cards when associated client stations are tracked in `AssocStore`.
- Reactively update the Scanner table whenever associations change by subscribing to `ctx.monitor.assoc.onChange(render)`.

## Capabilities

### Modified Capabilities
- `web-client`: Add requirements for Scanner proximity / client filtering and live connected station badge rendering on Access Point rows.

## Impact
- **Web UI**: `web/src/features/scanner/template.ts`, `web/src/features/scanner/mount.ts`, `web/src/features/scanner/filter.ts`.
- **Tests**: `web/test/features/ui-logic.test.ts`.
- **Documentation**: `docs/MONITORING.md`.
- **Wire/Core**: No wire protocol or firmware changes required; consumes existing `assocResult` telemetry and `AssocStore`.
