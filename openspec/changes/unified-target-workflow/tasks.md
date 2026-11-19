# Tasks

## 1. Handshake & Target Workspace UI Redesign

- [x] 1.1 Update `web/src/features/handshake/template.ts` with transparent handshake state display (`LISTENING`, `CRACKABLE (M1+M2)`, `FULL (4/4)`, `PMKID`), message badges (`M1`, `M2`, `M3`, `M4`), channel lock pill, and multi-target hopping advisory.
- [x] 1.2 Update `web/src/features/handshake/mount.ts` to enable export and copy actions as soon as `M1+M2` or `PMKID` is present, and wire 1-click channel lock toggle (`configureRadio`).
- [x] 1.3 Integrate active connected stations, probing clients (PNL), and deauth attack alerts directly into the target card.
- [x] 1.4 Add styling in `web/src/style.css` for the unified target card, channel lock pills, multi-target advisory, and message timeline.

## 2. Verification & Testing

- [x] 2.1 Run web unit tests (`pnpm --dir web test`) verifying crackable export behavior and channel locking logic.
- [x] 2.2 Verify production build (`pnpm --dir web run build`), Rust test suite (`cargo test`), and OpenSpec validation (`make spec`).
