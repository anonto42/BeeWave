use heapless::{String, Vec};

use super::types::{ProbeObservation, PROBE_CAPACITY};

pub const RSSI_THRESHOLD_DB: i8 = 6;
pub const DEDUP_INTERVAL_TICKS: u32 = 5;

struct ClientEntry {
    client: [u8; 6],
    ssid: String<32>,
    rssi: i8,
    last_tick: u32,
}

/// Bounded LRU deduplication table for client probe requests.
/// Bounded to `PROBE_CAPACITY` (32 entries, ~1.4 KB in RAM).
pub struct ProbeTracker {
    entries: Vec<ClientEntry, PROBE_CAPACITY>,
    tick: u32,
}

impl ProbeTracker {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            tick: 0,
        }
    }

    pub fn reset(&mut self) {
        self.entries.clear();
        self.tick = 0;
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    /// Feed one probe observation. Returns `Some(ProbeObservation)` if this report is novel or has
    /// significant signal change (>= 6 dB) or interval passed; returns `None` if it is a duplicate.
    pub fn observe(&mut self, obs: &ProbeObservation) -> Option<ProbeObservation> {
        if let Some(pos) = self
            .entries
            .iter()
            .position(|e| e.client == obs.client && e.ssid == obs.ssid)
        {
            let entry = &mut self.entries[pos];
            let rssi_delta = (entry.rssi - obs.rssi).abs();
            let elapsed = self.tick.wrapping_sub(entry.last_tick);
            if rssi_delta < RSSI_THRESHOLD_DB && elapsed < DEDUP_INTERVAL_TICKS {
                return None;
            }
            entry.rssi = obs.rssi;
            entry.last_tick = self.tick;
            return Some(obs.clone());
        }

        if self.entries.is_full() {
            if let Some(oldest) = self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.last_tick)
                .map(|(i, _)| i)
            {
                self.entries.swap_remove(oldest);
            }
        }
        let _ = self.entries.push(ClientEntry {
            client: obs.client,
            ssid: obs.ssid.clone(),
            rssi: obs.rssi,
            last_tick: self.tick,
        });
        Some(obs.clone())
    }
}

impl Default for ProbeTracker {
    fn default() -> Self {
        Self::new()
    }
}
