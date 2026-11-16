# Rust guide, learned from this codebase

Read the real code next to each point. Start with `crates/protocol`, then `crates/core`, last `firmware`.

## 1. Project shape
- A **crate** is a package (`Cargo.toml`). The root `Cargo.toml` is a **workspace** grouping `crates/protocol` and `crates/core`.
  `firmware/` is its own workspace because it builds for the ESP32 chip, not your PC.
- A **module** is a file or folder. `mod.rs` declares the files inside (`mod types;`) and `pub use` re-exports names so outside
  code writes `features::radio::RadioController` instead of the deep path. See `crates/core/src/features/radio/mod.rs`.

## 2. Data: structs and enums
- `struct` groups fields. `enum` is "one of these variants" and each variant can carry data:
  `DeviceCommand::ConfigureRadio { channel }` in `crates/protocol/src/types.rs`.
- `#[derive(Debug, Clone, PartialEq)]` asks the compiler to write printing, copying and `==` for you.
- `match` must handle every variant, so you cannot forget a case.

## 3. No exceptions: `Option` and `Result`
- `Option<T>` = `Some(value)` or `None`. `Result<T, E>` = `Ok(value)` or `Err(error)`.
- `?` returns the error early. `.map_err(..)` converts one error type to another (`crates/protocol/src/codec.rs`).
- We avoid `unwrap()` in device code because it would crash the chip; tests may use it.

## 4. Ownership and borrowing
- A value has one owner. `&T` borrows read-only, `&mut T` borrows to modify. `&self` / `&mut self` on a method says which.
- Look at `RadioController::handle(&mut self, cmd: &DeviceCommand)`: it changes itself but only reads the command.

## 5. Traits and generics (ports and adapters)
- A **trait** is a list of methods a type promises to have, like an interface: `crates/core/src/ports/radio.rs`.
- `struct RadioController<R: RadioDriver>` means "works with any `R` that implements `RadioDriver`".
  The firmware passes a real driver; `features/radio/tests.rs` passes a `FakeRadio`. Same code, no hardware needed.

## 6. `no_std` and `heapless`
- `#![no_std]` means no operating-system library, no automatic heap. Memory is precious on the chip.
- `heapless::String<32>` / `Vec<u8, 96>` / `FnvIndexMap` have a **fixed capacity** and never allocate.
  See `features/scan/table.rs` for what happens when the map is full.

## 7. Tests
- `#[cfg(test)] mod tests;` compiles `tests.rs` only for `cargo test`. Run all: `make test`. Run one crate: `cargo test -p walpad-core --features std`.
- Fakes (hand-written stand-ins for ports) live in the test files: `features/ota/tests.rs`.

## 8. `unsafe`
- Calling C code (ESP-IDF) is `unsafe` because the compiler cannot check it. All of it is confined to `firmware/src/adapters/`
  and `firmware/src/runtime/`, never in `crates/`.

## Suggested learning path
1. Read `types.rs` and `codec.rs` in `crates/protocol`; run `cargo test -p walpad-protocol --features std`.
2. Read `features/scan/` (smallest logic), then change `RSSI_DELTA` and watch a test fail.
3. Read `features/ota/state.rs` and `session.rs`: enums as state machines.
4. Read `app/service.rs`: how generics compose the pieces.
5. Only then look at `firmware/`.

Good free book: "The Rust Programming Language" (doc.rust-lang.org/book), chapters 3-10 cover everything above.
