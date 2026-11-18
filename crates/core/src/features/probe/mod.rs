//! Passive 802.11 client probe request sniffing.
//! Intercepts Probe Request management frames (subtype 0x40), extracting client MAC,
//! requested SSID (or empty string for wildcard), RSSI and channel with bounded deduplication.
mod parser;
mod tracker;
mod types;

pub use parser::parse_probe_request;
pub use tracker::ProbeTracker;
pub use types::{ProbeObservation, PROBE_CAPACITY};

#[cfg(test)]
mod tests;
