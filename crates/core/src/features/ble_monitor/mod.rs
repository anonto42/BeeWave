//! BLE advertisement monitor (receive-only): classifies advertisements, keeps per-class counts over the
//! last minute and notices when one class jumps far above its learned baseline. The firmware scans
//! passively (no scan requests are ever sent) and hands over the raw advertisement bytes.
mod classify;
pub mod flood;
mod monitor;
mod types;

pub use classify::classify;
pub use flood::{BleFloodDetector, BLE_FLOOD_COOLDOWN_TICKS, DEFAULT_BLE_FLOOD_THRESHOLD};
pub use monitor::BleMonitor;
pub use types::{BleCounts, BleSpikeHit, BleTick, STATS_EVERY_SECS, WARMUP_SECS, WINDOW_SECS};

#[cfg(test)]
mod tests;
