use beewave_protocol::TelemetryMessage;
use heapless::FnvIndexMap;

use super::types::{Observation, RSSI_DELTA, TABLE_CAP};

/// Remembers the last RSSI per BSSID in a fixed-size map (no heap allocation).
#[derive(Default)]
pub struct ScanTable {
    seen: FnvIndexMap<[u8; 6], i8, TABLE_CAP>,
}

impl ScanTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.seen.clear();
    }

    /// `Some(msg)` only when the network is new or its RSSI moved noticeably.
    pub fn observe(&mut self, o: Observation) -> Option<TelemetryMessage> {
        let report = match self.seen.get(&o.bssid) {
            Some(prev) => (o.rssi as i16 - *prev as i16).abs() >= RSSI_DELTA as i16,
            None => true,
        };
        if !report {
            return None;
        }
        // insert fails when the table is full: evict everything (cheap and bounded), then retry.
        if self.seen.insert(o.bssid, o.rssi).is_err() {
            self.seen.clear();
            let _ = self.seen.insert(o.bssid, o.rssi);
        }
        Some(TelemetryMessage::ScanResult {
            bssid: o.bssid,
            channel: o.channel,
            rssi: o.rssi,
            ssid: o.ssid,
        })
    }
}
