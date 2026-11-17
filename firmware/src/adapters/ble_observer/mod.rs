//! Passive BLE advertisement observer (receive-only). Runs a duty-cycled NimBLE scan while the owner has
//! switched the BLE monitor on, classifies each advertisement with the core's classifier and queues the
//! class for the app task. The scan is *passive*: it never sends a scan request, so nothing is transmitted.
//!
//! Off by default. BLE scanning shares the radio with the GATT link and the Wi-Fi sniffer, which is why it
//! is opt-in and scans only a quarter of the time (`SCAN_WINDOW_MS` of every `SCAN_INTERVAL_MS`).
mod observer;

pub use observer::{run, set_enabled};
