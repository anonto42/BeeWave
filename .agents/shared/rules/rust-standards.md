# Rust Standards

- `crates/*` are `#![no_std]` (tests may use `std`). Fixed-capacity collections via `heapless`; no `Vec`/`String`/`Box` in core.
- No `unwrap`/`expect`/`panic!` outside tests and `main` startup. Return typed errors (`RadioError`, `OtaError`, ...).
- A feature slice owns its state machine and its unit tests with fake ports; test invalid input and wrong-state paths, not just the happy path.
- `unsafe` only in `firmware/src/adapters/*` around ESP-IDF FFI, with a one-line justification.
- ESP-IDF callbacks (Wi-Fi promiscuous, BLE) must not allocate or block: write to a preallocated ring / `try_send`.
- Prefer `try_send`/bounded channels; never `send` into a queue a slow peer can fill.
- Format with `cargo fmt`; keep `cargo build -p walpad-core` (no_std) green.
