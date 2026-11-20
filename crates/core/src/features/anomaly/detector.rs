use beewave_protocol::{AnomalyKind, WifiSecurityProtocol};
use heapless::Vec;

use crate::features::scan::Observation;

use super::types::{
    kind_index, Anomaly, CAPACITY, COOLDOWN_TICKS, CYCLE_TICKS, KINDS, LEARN_TICKS,
    RETURN_AFTER_CYCLES, RSSI_JUMP_DB, TSF_ROLLBACK_THRESHOLD_US, TWO_CHANNEL_WINDOW_TICKS,
};

fn is_secure_protocol(proto: WifiSecurityProtocol) -> bool {
    matches!(
        proto,
        WifiSecurityProtocol::Wpa
            | WifiSecurityProtocol::Wpa2
            | WifiSecurityProtocol::Wpa3
            | WifiSecurityProtocol::Wpa2Wpa3
            | WifiSecurityProtocol::Enterprise
            | WifiSecurityProtocol::Owe
    )
}

struct Entry {
    bssid: [u8; 6],
    /// FNV-1a of the SSID; 0 for a hidden network (never compared).
    ssid: u32,
    channel: u8,
    /// Tick the BSSID was last heard on `channel`.
    channel_tick: u32,
    last_tick: u32,
    last_cycle: u32,
    /// RSSI when the BSSID was last seen in an earlier cycle.
    rssi: i8,
    security_proto: Option<WifiSecurityProtocol>,
    last_tsf: u64,
    /// Per kind: no report before this tick.
    quiet_until: [u32; KINDS],
}

/// Tracks up to `CAPACITY` BSSIDs (about 3 KB) and reports at most one finding per beacon.
pub struct AnomalyDetector {
    entries: Vec<Entry, CAPACITY>,
    start: Option<u32>,
}

impl AnomalyDetector {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            start: None,
        }
    }

    /// Feed one beacon. `tick` is the device's 1 Hz tick counter.
    pub fn observe(&mut self, o: &Observation, tick: u32) -> Option<Anomaly> {
        let learning = tick.saturating_sub(*self.start.get_or_insert(tick)) < LEARN_TICKS;
        let cycle = tick / CYCLE_TICKS;
        let ssid = ssid_hash(&o.ssid);
        let obs_proto = o.security.as_ref().map(|s| s.protocol);

        let Some(i) = self.entries.iter().position(|e| e.bssid == o.bssid) else {
            let matching_entry = if ssid != 0 {
                self.entries.iter().find(|e| e.ssid == ssid)
            } else {
                None
            };
            let is_downgrade = matching_entry.map_or(false, |e| {
                e.security_proto.map_or(false, is_secure_protocol)
                    && obs_proto.map_or(true, |p| {
                        matches!(p, WifiSecurityProtocol::Open | WifiSecurityProtocol::Wep)
                    })
            });
            let known_ssid = matching_entry.is_some();

            self.insert(Entry {
                bssid: o.bssid,
                ssid,
                channel: o.channel,
                channel_tick: tick,
                last_tick: tick,
                last_cycle: cycle,
                rssi: o.rssi,
                security_proto: obs_proto,
                last_tsf: o.tsf,
                quiet_until: [0; KINDS],
            });

            if !learning {
                if is_downgrade {
                    return Some(Anomaly {
                        kind: AnomalyKind::SecurityDowngrade,
                        bssid: o.bssid,
                        detail: 0,
                    });
                } else if known_ssid {
                    return Some(Anomaly {
                        kind: AnomalyKind::NewBssidForSsid,
                        bssid: o.bssid,
                        detail: 0,
                    });
                }
            }
            return None;
        };

        let e = &mut self.entries[i];
        let mut found: Option<(AnomalyKind, u8)> = None;

        if let (Some(known_proto), Some(new_proto)) = (e.security_proto, obs_proto) {
            if is_secure_protocol(known_proto)
                && matches!(
                    new_proto,
                    WifiSecurityProtocol::Open | WifiSecurityProtocol::Wep
                )
            {
                found = Some((AnomalyKind::SecurityDowngrade, 0));
            }
        }
        if obs_proto.is_some() {
            e.security_proto = obs_proto;
        }

        let missed = cycle.saturating_sub(e.last_cycle).saturating_sub(1);
        if missed > RETURN_AFTER_CYCLES {
            found = Some((AnomalyKind::Returned, missed.min(255) as u8));
        }
        if e.channel != o.channel {
            if found.is_none() && tick.saturating_sub(e.channel_tick) <= TWO_CHANNEL_WINDOW_TICKS {
                found = Some((AnomalyKind::TwoChannels, 0));
            }
            e.channel = o.channel;
        }
        e.channel_tick = tick;
        if e.last_cycle != cycle {
            let jump = (i16::from(o.rssi) - i16::from(e.rssi)).abs();
            if found.is_none() && jump > RSSI_JUMP_DB {
                found = Some((AnomalyKind::RssiOutlier, jump.min(255) as u8));
            }
            e.rssi = o.rssi;
            e.last_cycle = cycle;
        }
        if found.is_none() && e.ssid != 0 && ssid != 0 && e.ssid != ssid {
            found = Some((AnomalyKind::KarmaSsidChange, 0));
        }
        if found.is_none() && e.last_tsf > 0 && o.tsf < e.last_tsf {
            let rollback_us = e.last_tsf - o.tsf;
            if rollback_us >= TSF_ROLLBACK_THRESHOLD_US {
                let rollback_sec = (rollback_us / 1_000_000).min(255) as u8;
                found = Some((AnomalyKind::TsfRollback, rollback_sec.max(1)));
            }
        }
        e.last_tsf = o.tsf;
        e.last_tick = tick;
        e.ssid = ssid;

        let (kind, detail) = found?;
        let k = kind_index(kind);
        if learning || tick < e.quiet_until[k] {
            return None;
        }
        e.quiet_until[k] = tick + COOLDOWN_TICKS;
        Some(Anomaly {
            kind,
            bssid: o.bssid,
            detail,
        })
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

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// FNV-1a over the SSID bytes. Never 0 for a non-empty SSID, so 0 can mean "hidden".
fn ssid_hash(ssid: &str) -> u32 {
    if ssid.is_empty() {
        return 0;
    }
    let mut h: u32 = 0x811c_9dc5;
    for b in ssid.bytes() {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h.max(1)
}
