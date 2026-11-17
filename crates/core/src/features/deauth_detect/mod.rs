//! Deauthentication-frame detector (receive-only): turns one second of per-BSSID frame counts into
//! alerts. The sniffer adapter does the counting; this slice only decides what is an alert.
mod detector;
mod types;

pub use detector::DeauthDetector;
pub use types::{DeauthHit, DEFAULT_THRESHOLD, MAX_ALERTS};

#[cfg(test)]
mod tests;
