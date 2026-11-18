//! Passive WPA/WPA2 four-way handshake capture (receive-only). Parses EAPOL-Key frames the sniffer
//! already has access to while a scan is running and tracks which messages have been seen per
//! BSSID/client pair. Nothing is ever transmitted to cause or speed up a handshake.
//!
//! Also extracts PMKID Key Data Encapsulations from EAPOL-Key Message 1 when present, enabling
//! offline auditing without a full 4-way handshake capture.
mod parser;
mod pmkid;
mod tracker;
mod types;

pub use parser::parse_eapol;
pub use pmkid::extract_pmkid;
pub use tracker::HandshakeTracker;
pub use types::{Completed, EapolFrame, PmkidCapture, CAPACITY};

#[cfg(test)]
mod tests;
