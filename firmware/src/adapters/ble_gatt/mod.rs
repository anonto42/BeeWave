//! BLE GATT adapter (NimBLE).
//! Driving side: command writes become `Event::Command`.
//! Driven side: `BleSink` implements the core's `TelemetrySink` port; `run` pumps notifications.
//! NOTE: written against esp32-nimble 0.11.
mod server;
mod sink;
mod uuids;

pub use server::run;
pub use sink::BleSink;
