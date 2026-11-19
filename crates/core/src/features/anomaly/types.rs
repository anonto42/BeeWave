use beewave_protocol::AnomalyKind;

/// One scan cycle is about one 13-channel hop lap (13 x 250 ms), rounded to whole ticks (1 s each).
pub const CYCLE_TICKS: u32 = 4;
/// BSSIDs tracked at once; the least recently seen is dropped when full.
pub const CAPACITY: usize = 64;
/// A BSSID heard on two different channels within this many ticks is suspicious.
pub const TWO_CHANNEL_WINDOW_TICKS: u32 = 60;
/// An RSSI change larger than this between cycles is an outlier (dB).
pub const RSSI_JUMP_DB: i16 = 20;
/// A network missing for more than this many cycles and then heard again has "returned".
pub const RETURN_AFTER_CYCLES: u32 = 5;
/// The same kind for the same BSSID is reported at most this often.
pub const COOLDOWN_TICKS: u32 = 30;
/// Right after start every access point is new; nothing is reported for this long.
pub const LEARN_TICKS: u32 = 30;

/// A finding, ready to become an `AnomalyAlert`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Anomaly {
    pub kind: AnomalyKind,
    pub bssid: [u8; 6],
    /// Jump in dB (`RssiOutlier`) or cycles missed (`Returned`), capped at 255; 0 otherwise.
    pub detail: u8,
}

pub(super) const KINDS: usize = 6;

pub(super) const fn kind_index(k: AnomalyKind) -> usize {
    match k {
        AnomalyKind::NewBssidForSsid => 0,
        AnomalyKind::TwoChannels => 1,
        AnomalyKind::RssiOutlier => 2,
        AnomalyKind::Returned => 3,
        AnomalyKind::SecurityDowngrade => 4,
        AnomalyKind::KarmaSsidChange => 5,
    }
}
