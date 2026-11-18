use beewave_protocol::MAX_EAPOL_CAPTURE;

/// BSSID/client pairs tracked at once; the least recently seen is dropped when full.
pub const CAPACITY: usize = 32;

/// Distinct (BSSID, PMKID) pairs remembered to suppress duplicate `PmkidFound` telemetry.
/// 16 entries × 22 bytes = 352 bytes in RAM.
pub const PMKID_DEDUP_CAPACITY: usize = 16;

/// A classified EAPOL-Key frame (receive-only: nothing caused it to be sent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EapolFrame {
    pub bssid: [u8; 6],
    pub client: [u8; 6],
    /// 1-4: which of the four handshake messages this is.
    pub message: u8,
    pub replay_counter: u64,
    /// True if the real frame was longer than `MAX_EAPOL_CAPTURE` and `eapol` holds only the start of it.
    pub truncated: bool,
    /// The raw EAPOL frame (from its own version byte), capped to `MAX_EAPOL_CAPTURE`.
    pub eapol: heapless::Vec<u8, MAX_EAPOL_CAPTURE>,
}

/// A BSSID/client pair has enough messages for an offline cracking attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Completed {
    pub bssid: [u8; 6],
    pub client: [u8; 6],
}

/// A PMKID was passively extracted from an EAPOL-Key Message 1; not yet published.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PmkidCapture {
    pub bssid: [u8; 6],
    pub client: [u8; 6],
    pub pmkid: [u8; 16],
}
