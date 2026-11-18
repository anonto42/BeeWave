/// Bounded capacity for actively tracked client-AP associations.
pub const ASSOC_CAPACITY: usize = 32;

/// A parsed 802.11 Data frame association between a client station and an Access Point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssocObservation {
    pub bssid: [u8; 6],
    pub client: [u8; 6],
    pub channel: u8,
    pub rssi: i8,
}
