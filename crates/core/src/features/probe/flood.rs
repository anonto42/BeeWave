use beewave_protocol::TelemetryMessage;
use heapless::Vec;

pub const DEFAULT_PROBE_FLOOD_THRESHOLD: u16 = 30;
pub const MIN_UNIQUE_CLIENTS: usize = 8;
pub const PROBE_FLOOD_COOLDOWN_TICKS: u8 = 5;
pub const CLIENT_SAMPLE_CAP: usize = 32;

/// Zero-allocation rate-based probe request flood detector.
/// Tracks probe request frame velocity and unique client MACs in a 1-second window.
pub struct ProbeFloodDetector {
    threshold: u16,
    current_count: u16,
    unique_clients: Vec<[u8; 6], CLIENT_SAMPLE_CAP>,
    cooldown: u8,
}

impl ProbeFloodDetector {
    pub const fn new() -> Self {
        Self {
            threshold: DEFAULT_PROBE_FLOOD_THRESHOLD,
            current_count: 0,
            unique_clients: Vec::new(),
            cooldown: 0,
        }
    }

    pub fn threshold(&self) -> u16 {
        self.threshold
    }

    pub fn set_threshold(&mut self, threshold: u16) {
        self.threshold = threshold;
    }

    pub fn reset(&mut self) {
        self.current_count = 0;
        self.unique_clients.clear();
        self.cooldown = 0;
    }

    /// Record a received probe request frame and its client MAC.
    pub fn observe(&mut self, client: &[u8; 6]) {
        self.current_count = self.current_count.saturating_add(1);
        if !self.unique_clients.contains(client) && !self.unique_clients.is_full() {
            let _ = self.unique_clients.push(*client);
        }
    }

    /// Evaluates the 1-second window on tick. Returns `Some(TelemetryMessage::ProbeFloodAlert)` if
    /// the probe rate meets or exceeds the threshold with sufficient unique clients and cooldown has elapsed.
    pub fn tick(&mut self) -> Option<TelemetryMessage> {
        let count = self.current_count;
        let unique = self.unique_clients.len() as u16;

        self.current_count = 0;
        self.unique_clients.clear();

        if self.cooldown > 0 {
            self.cooldown -= 1;
        }

        if self.threshold > 0 && count >= self.threshold && unique >= (MIN_UNIQUE_CLIENTS as u16) {
            if self.cooldown == 0 {
                self.cooldown = PROBE_FLOOD_COOLDOWN_TICKS;
                return Some(TelemetryMessage::ProbeFloodAlert {
                    rate: count,
                    unique_clients: unique,
                });
            }
        }

        None
    }
}

impl Default for ProbeFloodDetector {
    fn default() -> Self {
        Self::new()
    }
}
