use heapless::Vec;

use super::types::{AssocObservation, ASSOC_CAPACITY};

const DEDUP_RSSI_DELTA: i8 = 6;
const DEDUP_INTERVAL_SECS: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssocEntry {
    pub bssid: [u8; 6],
    pub client: [u8; 6],
    pub channel: u8,
    pub rssi: i8,
    pub last_seen_sec: u32,
    pub frame_count: u32,
}

/// Bounded LRU tracker for client-to-AP associations.
pub struct AssocTracker {
    entries: Vec<AssocEntry, ASSOC_CAPACITY>,
    current_sec: u32,
}

impl AssocTracker {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_sec: 0,
        }
    }

    /// Advance the internal clock by 1 second.
    pub fn tick(&mut self) {
        self.current_sec = self.current_sec.saturating_add(1);
    }

    /// Observe an association frame. Returns `Some(obs)` if this is a new pair or
    /// meets deduplication thresholds (6 dB delta or 5 seconds elapsed).
    pub fn observe(&mut self, obs: &AssocObservation) -> Option<AssocObservation> {
        let current_sec = self.current_sec;
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.bssid == obs.bssid && e.client == obs.client)
        {
            entry.channel = obs.channel;
            entry.frame_count = entry.frame_count.saturating_add(1);

            let rssi_delta = (i16::from(obs.rssi) - i16::from(entry.rssi)).abs();
            let time_elapsed = current_sec.saturating_sub(entry.last_seen_sec);

            if rssi_delta >= i16::from(DEDUP_RSSI_DELTA) || time_elapsed >= DEDUP_INTERVAL_SECS {
                entry.rssi = obs.rssi;
                entry.last_seen_sec = current_sec;
                Some(*obs)
            } else {
                None
            }
        } else {
            if self.entries.is_full() {
                // Evict the entry with the oldest last_seen_sec
                if let Some((oldest_idx, _)) = self
                    .entries
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, e)| e.last_seen_sec)
                {
                    self.entries.swap_remove(oldest_idx);
                }
            }

            let entry = AssocEntry {
                bssid: obs.bssid,
                client: obs.client,
                channel: obs.channel,
                rssi: obs.rssi,
                last_seen_sec: current_sec,
                frame_count: 1,
            };
            let _ = self.entries.push(entry);
            Some(*obs)
        }
    }

    /// Number of tracked associations.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether tracker is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for AssocTracker {
    fn default() -> Self {
        Self::new()
    }
}
