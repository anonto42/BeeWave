use heapless::Vec;

use super::types::{Completed, EapolFrame, CAPACITY};

struct Entry {
    bssid: [u8; 6],
    client: [u8; 6],
    /// Bit (message - 1) set once that message has been seen.
    seen: u8,
    reported: bool,
    last_tick: u32,
}

/// Which messages have been seen for each BSSID/client pair, bounded to `CAPACITY` pairs (about 1 KB).
/// Reports completion once a pair has an ANonce-carrying message (1 or 3) and the SNonce+MIC message (2).
pub struct HandshakeTracker {
    entries: Vec<Entry, CAPACITY>,
    tick: u32,
}

const ANONCE_BITS: u8 = (1 << 0) | (1 << 2); // message 1 or message 3
const SNONCE_BIT: u8 = 1 << 1; // message 2

impl HandshakeTracker {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            tick: 0,
        }
    }

    /// Forget every pair (called when the listener is switched on, like the BLE monitor).
    pub fn reset(&mut self) {
        self.entries.clear();
        self.tick = 0;
    }

    /// Feed one classified frame. Returns `Some` the moment this pair becomes complete (once per pair).
    pub fn observe(&mut self, f: &EapolFrame) -> Option<Completed> {
        self.tick = self.tick.wrapping_add(1);
        let bit = 1u8 << (f.message - 1);

        let i = match self
            .entries
            .iter()
            .position(|e| e.bssid == f.bssid && e.client == f.client)
        {
            Some(i) => i,
            None => {
                self.insert(Entry {
                    bssid: f.bssid,
                    client: f.client,
                    seen: 0,
                    reported: false,
                    last_tick: self.tick,
                });
                self.entries.len() - 1
            }
        };
        let e = &mut self.entries[i];
        e.seen |= bit;
        e.last_tick = self.tick;
        let complete = e.seen & ANONCE_BITS != 0 && e.seen & SNONCE_BIT != 0;
        if complete && !e.reported {
            e.reported = true;
            return Some(Completed {
                bssid: f.bssid,
                client: f.client,
            });
        }
        None
    }

    fn insert(&mut self, entry: Entry) {
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
        let _ = self.entries.push(entry);
    }
}

impl Default for HandshakeTracker {
    fn default() -> Self {
        Self::new()
    }
}
