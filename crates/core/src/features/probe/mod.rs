//! Passive 802.11 client probe request sniffing.
//! Intercepts Probe Request management frames (subtype 0x40), extracting client MAC,
//! requested SSID (or empty string for wildcard), RSSI and channel with bounded deduplication.
pub mod flood;
mod parser;
mod tracker;
mod types;

pub use flood::{
    ProbeFloodDetector, DEFAULT_PROBE_FLOOD_THRESHOLD, MIN_UNIQUE_CLIENTS,
    PROBE_FLOOD_COOLDOWN_TICKS,
};
pub use parser::parse_probe_request;
pub use tracker::ProbeTracker;
pub use types::{ProbeObservation, PROBE_CAPACITY};

#[cfg(test)]
mod tests;
