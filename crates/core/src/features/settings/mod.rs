//! Settings: validate, persist and report the owner's settings (scan, radio, telemetry, alerts).
mod service;
mod validate;

pub use service::SettingsService;
pub use validate::{
    hop_order, validate, CHANNEL_MASK_BITS, DEAUTH_THRESHOLD, DEAUTH_WINDOW_MS, DEDUP_RSSI_DB,
    HOP_DWELL_MS, SCAN_INTERVAL_MS, STATUS_PERIOD_MS,
};

#[cfg(test)]
mod tests;
