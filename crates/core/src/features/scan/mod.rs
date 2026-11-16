//! Scan reporting: de-duplicate sniffed beacons so BLE only carries new or changed networks.
mod table;
mod types;

pub use table::ScanTable;
pub use types::{Observation, RSSI_DELTA, TABLE_CAP};

#[cfg(test)]
mod tests;
