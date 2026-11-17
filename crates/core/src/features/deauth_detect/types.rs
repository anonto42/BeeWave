/// Alerts at the default of 10 deauth frames per second from one BSSID.
pub const DEFAULT_THRESHOLD: u8 = 10;
/// At most this many BSSIDs are reported per second (the strongest ones), so a storm cannot flood telemetry.
pub const MAX_ALERTS: usize = 4;

/// One BSSID that reached the threshold within the one-second window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeauthHit {
    pub bssid: [u8; 6],
    pub count: u16,
}
