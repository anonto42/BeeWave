//! BLE GATT adapter (NimBLE).
//! Driving side: command writes become `Event::Command`.
//! Driven side: `BleSink` implements the core's `TelemetrySink` port; `run` pumps notifications.
//! NOTE: written against esp32-nimble 0.11.
mod coex_diag;
mod names;
mod pairing;
mod server;
mod sink;
mod tuning;
mod uuids;

pub use server::{is_ready, run};
pub use sink::BleSink;
