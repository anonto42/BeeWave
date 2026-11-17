## 1. Protocol
- [x] 1.1 Append `Provision { key }` command and `Unclaimed` telemetry; golden vectors
- [x] 1.2 Mirror in `web/src/shared/protocol`

## 2. Core
- [x] 2.1 `AuthGate::claim`, `SettingsStore` key methods, host tests (claim once, wrong state, storage failure)
- [x] 2.2 `App`: `Unclaimed` reply, `Provision` handling, key loading at start

## 3. Firmware
- [x] 3.1 NVS key blob in `NvsSettings`; `config.rs` treats a missing `BEEWAVE_KEY` as "unclaimed"
- [x] 3.2 Makefile: unclaimed by default, `FW_KEY=file`, `make fw-unclaim`

## 4. Web
- [x] 4.1 Authenticator claims on `Unclaimed` and saves the key after `AuthResult ok`
- [x] 4.2 Mock transport models the unclaimed device; UI wording

## 5. Docs
- [x] 5.1 AGENTS.md invariant 5, USAGE, TESTING, BENCH_RUNBOOK, ARCHITECTURE
