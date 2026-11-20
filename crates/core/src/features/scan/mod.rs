//! Scan reporting: parse sniffed beacons and de-duplicate them so BLE only carries new or changed networks.
pub mod flood;
mod frame;
pub mod phy;
pub mod security;
mod table;
mod types;
pub mod wps;

pub use flood::{BeaconFloodDetector, DEFAULT_BEACON_FLOOD_THRESHOLD, MIN_UNIQUE_BSSIDS};
pub use frame::parse_beacon;
pub use phy::{classify_phy, PhyInfo, PhyTracker};
pub use security::{classify_security, parse_rsn_element, SecurityInfo, SecurityTracker};
pub use table::ScanTable;
pub use types::{Observation, RSSI_DELTA, TABLE_CAP};
pub use wps::{parse_wps_element, WpsInfo, WpsTracker};

#[cfg(test)]
mod tests;
