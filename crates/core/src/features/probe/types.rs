use heapless::String;

/// Number of active client probe observations tracked in memory at once.
pub const PROBE_CAPACITY: usize = 32;

/// A parsed 802.11 Probe Request observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeObservation {
    /// Transmitter client MAC address.
    pub client: [u8; 6],
    pub channel: u8,
    pub rssi: i8,
    /// Requested SSID; empty if wildcard/broadcast probe.
    pub ssid: String<32>,
}
