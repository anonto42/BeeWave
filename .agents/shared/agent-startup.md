# Agent Startup Checklist

1. **Environment:** `nix develop` (or `direnv allow`), then `make doctor`. Firmware needs `make setup` once (espup).
2. **OpenSpec:** read `openspec/config.yaml`, run `openspec list` and `openspec list --specs`, check `openspec/changes/`.
   Non-trivial change -> proposal under `openspec/changes/<name>/`, must pass `make spec`.
3. **Architecture:** read `docs/ARCHITECTURE.md`. New capability = new slice in `crates/core/src/features/<name>/` + adapter in
   `firmware/src/adapters/` + web slice in `web/src/features/<name>/`. Compose only in `crates/core/src/app/`, `main.rs`, `web/src/main.ts`.
4. **Code map:** `graphify-out/GRAPH_REPORT.md` (regenerate with `make graphify`) before wide refactors.
5. **Device key:** `make fw-key` creates the untracked `firmware/.beewave-key` (firmware targets use it); `make fw-ota-key` does the same for the OTA signing key (`.beewave-ota.pem`/`.pub`); never commit either.
6. **Verify before & after:** `make check` (fmt, host tests, no_std build, web build, specs). Hardware: `make fw-build`, then `make fw-run`.
7. **Hardware facts:** board on `/dev/ttyACM0` (CH9102), ESP32 rev1 4 MB; hold BOOT if auto-reset fails.
8. **Commits:** follow `.agents/shared/rules/git.md` (separate logical commits, conventional messages, no AI attribution, device git user).
