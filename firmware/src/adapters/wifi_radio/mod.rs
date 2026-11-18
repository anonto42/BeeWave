//! Wi-Fi radio adapter: promiscuous sniffer + channel hopper.
//! Driven side: implements the core's `RadioDriver` port.
//! Driving side: the Wi-Fi callback parses beacons and pushes `Observation`s into the frame queue,
//! counts deauthentication frames into lock-free counters (`deauth.rs`), and -- only while the owner has
//! switched it on -- classifies EAPOL-Key frames into the handshake queue. All receive-only.
//!
//! Files: `driver.rs` (WifiRadio), `callback.rs` (runs in the Wi-Fi task: must not block or allocate),
//! `hopper.rs` (channel-hopping thread), `shared.rs` (state shared between them).
mod callback;
pub use callback::{install_handshake, set_handshake_capture};
pub mod deauth;
mod driver;
mod hopper;
mod shared;

pub use driver::WifiRadio;
