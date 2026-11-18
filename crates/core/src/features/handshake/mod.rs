//! Passive WPA/WPA2 four-way handshake capture (receive-only). Parses EAPOL-Key frames the sniffer
//! already has access to while a scan is running and tracks which messages have been seen per
//! BSSID/client pair. Nothing is ever transmitted to cause or speed up a handshake.
mod parser;
mod tracker;
mod types;

pub use parser::parse_eapol;
pub use tracker::HandshakeTracker;
pub use types::{Completed, EapolFrame, CAPACITY};

#[cfg(test)]
mod tests;
