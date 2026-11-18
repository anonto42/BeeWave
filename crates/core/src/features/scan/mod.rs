//! Scan reporting: parse sniffed beacons and de-duplicate them so BLE only carries new or changed networks.
mod frame;
mod table;
mod types;
pub mod wps;

pub use frame::parse_beacon;
pub use table::ScanTable;
pub use types::{Observation, RSSI_DELTA, TABLE_CAP};
pub use wps::{parse_wps_element, WpsInfo, WpsTracker};

#[cfg(test)]
mod tests;
