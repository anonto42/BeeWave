use beewave_protocol::{AdClass, TelemetryMessage};

use super::types::{class_index, ALL_CLASSES, CLASSES};

pub const DEFAULT_BLE_FLOOD_THRESHOLD: u16 = 30;
pub const BLE_FLOOD_COOLDOWN_TICKS: u8 = 5;

/// Zero-allocation sliding 1-second rate detector for BLE advertisement flooding / spam attacks.
pub struct BleFloodDetector {
    threshold: u16,
    counts: [u16; CLASSES],
    cooldown: u8,
}

impl BleFloodDetector {
    pub const fn new() -> Self {
        Self {
            threshold: DEFAULT_BLE_FLOOD_THRESHOLD,
            counts: [0; CLASSES],
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
        self.counts = [0; CLASSES];
        self.cooldown = 0;
    }

    /// Record a received advertisement of the given class.
    pub fn observe(&mut self, class: AdClass) {
        let idx = class_index(class);
        self.counts[idx] = self.counts[idx].saturating_add(1);
    }

    /// Advance one second on tick. Returns `Some(TelemetryMessage::BleFloodAlert)` if
    /// the aggregate rate meets or exceeds the threshold and cooldown has elapsed.
    pub fn tick(&mut self) -> Option<TelemetryMessage> {
        let total: u16 = self.counts.iter().fold(0u16, |acc, &c| acc.saturating_add(c));

        // Identify dominant class
        let mut max_count = 0u16;
        let mut dominant = AdClass::Generic;
        for (i, &cnt) in self.counts.iter().enumerate() {
            if cnt > max_count {
                max_count = cnt;
                dominant = ALL_CLASSES[i];
            }
        }

        self.counts = [0; CLASSES];

        if self.cooldown > 0 {
            self.cooldown -= 1;
        }

        if self.threshold > 0 && total >= self.threshold {
            if self.cooldown == 0 {
                self.cooldown = BLE_FLOOD_COOLDOWN_TICKS;
                return Some(TelemetryMessage::BleFloodAlert {
                    rate: total,
                    class: dominant,
                });
            }
        }

        None
    }
}

impl Default for BleFloodDetector {
    fn default() -> Self {
        Self::new()
    }
}
