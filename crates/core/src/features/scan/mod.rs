//! Scan feature: dedupe sniffed beacons so BLE only carries new/changed networks.
use heapless::{FnvIndexMap, String};
use walpad_protocol::TelemetryMessage;

pub const TABLE_CAP: usize = 64; // power of two (FnvIndexMap)
pub const RSSI_DELTA: i8 = 6;

/// What a driving adapter (sniffer) hands to the core.
#[derive(Debug, Clone)]
pub struct Observation {
    pub bssid: [u8; 6],
    pub channel: u8,
    pub rssi: i8,
    pub ssid: String<32>,
}

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

    /// Some(msg) only when the network is new or its RSSI moved noticeably.
    pub fn observe(&mut self, o: Observation) -> Option<TelemetryMessage> {
        let report = match self.seen.get(&o.bssid) {
            Some(prev) => (o.rssi as i16 - *prev as i16).abs() >= RSSI_DELTA as i16,
            None => true,
        };
        if !report {
            return None;
        }
        if self.seen.insert(o.bssid, o.rssi).is_err() {
            self.seen.clear(); // full: evict everything, cheap and bounded
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

#[cfg(test)]
mod tests {
    use super::*;
    fn obs(rssi: i8) -> Observation {
        Observation {
            bssid: [1; 6],
            channel: 1,
            rssi,
            ssid: "x".try_into().unwrap(),
        }
    }
    #[test]
    fn dedupes_until_rssi_moves() {
        let mut t = ScanTable::new();
        assert!(t.observe(obs(-50)).is_some());
        assert!(t.observe(obs(-52)).is_none());
        assert!(t.observe(obs(-60)).is_some());
    }
    #[test]
    fn survives_overflow() {
        let mut t = ScanTable::new();
        for i in 0..200u8 {
            let mut o = obs(-50);
            o.bssid = [i, 0, 0, 0, 0, 1];
            assert!(t.observe(o).is_some());
        }
    }
}
