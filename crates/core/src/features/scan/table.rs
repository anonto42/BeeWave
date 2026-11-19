use beewave_protocol::{ScanSettings, TelemetryMessage};
use heapless::FnvIndexMap;

use super::types::{Observation, RSSI_DELTA, TABLE_CAP};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableEntry {
    pub rssi: i8,
    pub reported_at: u32,
    pub had_empty_ssid: bool,
}

/// Remembers the last report per BSSID in a fixed-size map (no heap allocation) and decides which
/// observations are worth reporting under the owner's scan settings.
pub struct ScanTable {
    /// Tracking state per BSSID.
    seen: FnvIndexMap<[u8; 6], TableEntry, TABLE_CAP>,
    dedup_db: i16,
    min_interval_ms: u32,
    ssid_filter: heapless::String<32>,
    channel_mask: u16,
}

impl Default for ScanTable {
    fn default() -> Self {
        Self {
            seen: FnvIndexMap::new(),
            dedup_db: i16::from(RSSI_DELTA),
            min_interval_ms: 0,
            ssid_filter: heapless::String::new(),
            channel_mask: 0,
        }
    }
}

impl ScanTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.seen.clear();
    }

    /// Adopt the owner's scan settings. Takes effect on the next observation.
    pub fn configure(&mut self, s: &ScanSettings) {
        self.dedup_db = i16::from(s.dedup_rssi_db);
        self.min_interval_ms = s.scan_interval_ms;
        self.ssid_filter = s.ssid_filter.clone();
        self.channel_mask = s.channel_filter_mask;
    }

    /// `Some(msg)` when the beacon passes the filters and the network is new, uncloaked,
    /// or its RSSI moved by at least the dedup threshold and the minimum interval has passed.
    /// `now_ms` is the device's running time in milliseconds.
    pub fn observe(&mut self, o: Observation, now_ms: u32) -> Option<TelemetryMessage> {
        if !self.passes_filters(&o) {
            return None;
        }

        let is_empty = o.ssid.is_empty();
        let (report, decloaked, had_empty) = match self.seen.get(&o.bssid) {
            Some(prev) => {
                if prev.had_empty_ssid && !is_empty {
                    // Hidden network revealed with cleartext SSID: bypass dedup
                    (true, true, false)
                } else {
                    let eligible = now_ms.wrapping_sub(prev.reported_at) >= self.min_interval_ms
                        && (i16::from(o.rssi) - i16::from(prev.rssi)).abs() >= self.dedup_db;
                    let preserve_empty = prev.had_empty_ssid && is_empty;
                    (eligible, false, preserve_empty)
                }
            }
            None => (true, false, is_empty),
        };

        if !report {
            return None;
        }

        let entry = TableEntry {
            rssi: o.rssi,
            reported_at: now_ms,
            had_empty_ssid: had_empty,
        };

        // insert fails when the table is full: evict everything (cheap and bounded), then retry.
        if self.seen.insert(o.bssid, entry).is_err() {
            self.seen.clear();
            let _ = self.seen.insert(o.bssid, entry);
        }

        if decloaked {
            Some(TelemetryMessage::DecloakedResult {
                bssid: o.bssid,
                ssid: o.ssid,
                channel: o.channel,
                rssi: o.rssi,
            })
        } else {
            Some(TelemetryMessage::ScanResult {
                bssid: o.bssid,
                channel: o.channel,
                rssi: o.rssi,
                ssid: o.ssid,
            })
        }
    }

    fn passes_filters(&self, o: &Observation) -> bool {
        // mask bit 0 = channel 1; no bits set = every channel
        let channel_ok = self.channel_mask == 0
            || (1..=13).contains(&o.channel) && self.channel_mask & (1 << (o.channel - 1)) != 0;
        channel_ok && contains_ignore_case(&o.ssid, &self.ssid_filter)
    }
}

/// ASCII case-insensitive substring test; an empty needle matches everything (including hidden networks).
fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    needle.is_empty()
        || haystack
            .as_bytes()
            .windows(needle.len())
            .any(|w| w.eq_ignore_ascii_case(needle.as_bytes()))
}
