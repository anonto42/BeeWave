use beewave_protocol::{ScanSettings, TelemetryMessage};
use heapless::FnvIndexMap;

use super::types::{Observation, RSSI_DELTA, TABLE_CAP};

/// Remembers the last report per BSSID in a fixed-size map (no heap allocation) and decides which
/// observations are worth reporting under the owner's scan settings.
pub struct ScanTable {
    /// RSSI and the time (ms) of the last report, per BSSID.
    seen: FnvIndexMap<[u8; 6], (i8, u32), TABLE_CAP>,
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

    /// `Some(msg)` when the beacon passes the filters and the network is new, or its RSSI moved by at
    /// least the dedup threshold and the minimum interval since its last report has passed.
    /// `now_ms` is the device's running time in milliseconds.
    pub fn observe(&mut self, o: Observation, now_ms: u32) -> Option<TelemetryMessage> {
        if !self.passes_filters(&o) {
            return None;
        }
        let report = match self.seen.get(&o.bssid) {
            Some(&(prev, at)) => {
                now_ms.wrapping_sub(at) >= self.min_interval_ms
                    && (i16::from(o.rssi) - i16::from(prev)).abs() >= self.dedup_db
            }
            None => true,
        };
        if !report {
            return None;
        }
        // insert fails when the table is full: evict everything (cheap and bounded), then retry.
        if self.seen.insert(o.bssid, (o.rssi, now_ms)).is_err() {
            self.seen.clear();
            let _ = self.seen.insert(o.bssid, (o.rssi, now_ms));
        }
        Some(TelemetryMessage::ScanResult {
            bssid: o.bssid,
            channel: o.channel,
            rssi: o.rssi,
            ssid: o.ssid,
        })
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
