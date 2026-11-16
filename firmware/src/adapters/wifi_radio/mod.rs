//! Wi-Fi radio adapter: promiscuous sniffer + channel hopper.
//! Driven side: implements the core's `RadioDriver` port.
//! Driving side: the Wi-Fi callback parses beacons and pushes `Observation`s into the frame queue.
//!
//! Files: `driver.rs` (WifiRadio), `callback.rs` (runs in the Wi-Fi task: must not block or allocate),
//! `hopper.rs` (channel-hopping thread), `shared.rs` (state shared between them).
mod callback;
mod driver;
mod hopper;
mod shared;

pub use driver::WifiRadio;
