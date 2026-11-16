# File Structure Rules

One concern per file. A reader should know what a file holds from its name.

## TypeScript (`web/`)
- `types.ts`: types and interfaces only (no runtime code). `constants.ts`: constants only.
- One class per file, named in kebab-case after the class (`ble-transport.ts` -> `BleTransport`).
- Pure functions grouped by purpose (`decode.ts`, `encode.ts`), HTML in `template.ts`, wiring in `mount.ts`.
- Every folder has an `index.ts` that re-exports the public API; import a folder, not its internals.
- Features import `shared/` only, never another feature. Tests mirror the source path under `test/`.

## Rust (`crates/`, `firmware/`)
- `mod.rs` / `lib.rs` only declare modules and `pub use` re-exports (a table of contents, no logic).
- `types.rs` (data + constants), `error.rs` (error enums), `state.rs` (state-machine enums), then one file per behaviour
  (`controller.rs`, `table.rs`, `session.rs`, `codec.rs`).
- One trait (port) per file under `ports/`.
- Tests live in a sibling `tests.rs` declared with `#[cfg(test)] mod tests;`, using fakes for ports.
- Magic numbers become named constants next to the types they belong to.
- Public names are re-exported from the folder's `mod.rs` so callers write `features::radio::RadioController`.
- Keep files small (aim under ~120 lines); split when a file grows a second reason to change.
