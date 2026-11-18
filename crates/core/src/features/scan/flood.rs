use beewave_protocol::TelemetryMessage;
use heapless::Vec;

pub const DEFAULT_BEACON_FLOOD_THRESHOLD: u16 = 25;
pub const MIN_UNIQUE_BSSIDS: usize = 8;
pub const FLOOD_COOLDOWN_TICKS: u8 = 5;
pub const BSSID_SAMPLE_CAP: usize = 32;

/// Zero-allocation rate-based beacon flood detector.
/// Tracks beacon packet velocity and unique BSSIDs in a 1-second window.
pub struct BeaconFloodDetector {
    threshold: u16,
    current_count: u16,
    unique_bssids: Vec<[u8; 6], BSSID_SAMPLE_CAP>,
    cooldown: u8,
}

impl BeaconFloodDetector {
    pub const fn new() -> Self {
        Self {
            threshold: DEFAULT_BEACON_FLOOD_THRESHOLD,
            current_count: 0,
            unique_bssids: Vec::new(),
            cooldown: 0,
        }
    }

    pub fn threshold(&self) -> u16 {
        self.threshold
    }

    pub fn set_threshold(&mut self, threshold: u16) {
        self.threshold = threshold;
    }

    /// Record a received beacon frame and its BSSID.
    pub fn observe(&mut self, bssid: &[u8; 6]) {
        self.current_count = self.current_count.saturating_add(1);
        if !self.unique_bssids.contains(bssid) && !self.unique_bssids.is_full() {
            let _ = self.unique_bssids.push(*bssid);
        }
    }

    /// Evaluates the 1-second window on tick. Returns `Some(TelemetryMessage::BeaconFloodAlert)` if
    /// the beacon rate meets or exceeds the threshold with sufficient unique BSSIDs and cooldown has elapsed.
    pub fn tick(&mut self) -> Option<TelemetryMessage> {
        let count = self.current_count;
        let unique = self.unique_bssids.len() as u16;

        self.current_count = 0;
        self.unique_bssids.clear();

        if self.cooldown > 0 {
            self.cooldown -= 1;
        }

        if self.threshold > 0 && count >= self.threshold && unique >= (MIN_UNIQUE_BSSIDS as u16) {
            if self.cooldown == 0 {
                self.cooldown = FLOOD_COOLDOWN_TICKS;
                return Some(TelemetryMessage::BeaconFloodAlert {
                    rate: count,
                    unique_bssids: unique,
                });
            }
        }

        None
    }
}

impl Default for BeaconFloodDetector {
    fn default() -> Self {
        Self::new()
    }
}
