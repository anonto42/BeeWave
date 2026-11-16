//! Feature slices. Each folder owns its types, logic and tests, and depends only on
//! `ports` and `walpad_protocol`, never on another feature.
//!
//! Folder layout (same in every slice):
//! - `mod.rs`: declares the files and re-exports the public names (table of contents only)
//! - `types.rs` / `state.rs`: data types and constants
//! - `<logic>.rs`: the behaviour
//! - `tests.rs`: unit tests with fake ports
pub mod ota;
pub mod radio;
pub mod scan;
pub mod status;
