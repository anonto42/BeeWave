use heapless::Vec;

use super::types::{DeauthHit, DEFAULT_THRESHOLD, MAX_ALERTS};

/// Compares per-BSSID deauth counts of one second against a threshold. Holds no frame state: the
/// window is the interval between two calls to `evaluate`, owned by whoever drains the counters.
pub struct DeauthDetector {
    threshold: u8,
}

impl DeauthDetector {
    pub const fn new() -> Self {
        Self {
            threshold: DEFAULT_THRESHOLD,
        }
    }

    pub fn threshold(&self) -> u8 {
        self.threshold
    }

    /// Frames per second that raise an alert; 0 turns the detector off.
    pub fn set_threshold(&mut self, threshold: u8) {
        self.threshold = threshold;
    }

    /// The BSSIDs at or above the threshold in this one-second window, strongest first, at most
    /// `MAX_ALERTS`. Nothing is reported while the detector is off.
    pub fn evaluate(&self, counts: &[([u8; 6], u16)]) -> Vec<DeauthHit, MAX_ALERTS> {
        self.evaluate_window(counts, 1000)
    }

    /// Like `evaluate`, for counts gathered over `window_ms`: the rate in frames per second is compared with
    /// the threshold and reported as `count`.
    pub fn evaluate_window(
        &self,
        counts: &[([u8; 6], u16)],
        window_ms: u16,
    ) -> Vec<DeauthHit, MAX_ALERTS> {
        let mut hits: Vec<DeauthHit, MAX_ALERTS> = Vec::new();
        if self.threshold == 0 || window_ms == 0 {
            return hits;
        }
        for &(bssid, frames) in counts {
            let rate =
                (u32::from(frames) * 1000 / u32::from(window_ms)).min(u32::from(u16::MAX)) as u16;
            if rate < u16::from(self.threshold) {
                continue;
            }
            let count = rate;
            let hit = DeauthHit { bssid, count };
            if hits.is_full() {
                // replace the weakest kept hit if this one is stronger
                if let Some((i, weakest)) = hits.iter().enumerate().min_by_key(|(_, h)| h.count) {
                    if count > weakest.count {
                        hits[i] = hit;
                    }
                }
            } else {
                let _ = hits.push(hit);
            }
        }
        hits.sort_unstable_by(|a, b| b.count.cmp(&a.count));
        hits
    }
}

impl Default for DeauthDetector {
    fn default() -> Self {
        Self::new()
    }
}
