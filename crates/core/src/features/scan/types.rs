use heapless::String;

use super::phy::PhyInfo;
use super::security::SecurityInfo;
use super::wps::WpsInfo;

/// Table capacity. Must be a power of two (required by `FnvIndexMap`).
pub const TABLE_CAP: usize = 64;
/// Report a known network again only if its RSSI moved at least this many dB.
pub const RSSI_DELTA: i8 = 6;

/// What a driving adapter (the sniffer) hands to the core.
#[derive(Debug, Clone)]
pub struct Observation {
    pub bssid: [u8; 6],
    pub channel: u8,
    pub rssi: i8,
    pub ssid: String<32>,
    pub wps: Option<WpsInfo>,
    pub security: Option<SecurityInfo>,
    pub phy: Option<PhyInfo>,
    pub tsf: u64,
}
