use beewave_protocol::AdClass;

/// Counts cover the last minute.
pub const WINDOW_SECS: usize = 60;
/// `BleStats` is published this often while the monitor runs.
pub const STATS_EVERY_SECS: u32 = 10;
/// No spike can fire until the baseline had this long to form.
pub const WARMUP_SECS: u32 = 60;
/// A spike must exceed the baseline by at least this many advertisements, whatever the factor.
pub const MIN_SPIKE_DELTA: u16 = 20;
/// After a spike for a class, wait this long before reporting that class again.
pub const SPIKE_COOLDOWN_SECS: u32 = 60;
/// Number of classes (the `AdClass` variants).
pub const CLASSES: usize = 4;

/// Advertisements per class over the last minute, in `AdClass` order.
pub type BleCounts = [u16; CLASSES];

/// A class far above its baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BleSpikeHit {
    pub class: AdClass,
    pub count: u16,
    pub baseline: u16,
}

/// What one second of the monitor produced.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BleTick {
    /// Present every `STATS_EVERY_SECS`.
    pub stats: Option<BleCounts>,
    pub spike: Option<BleSpikeHit>,
}

/// Index of a class in `BleCounts`.
pub const fn class_index(c: AdClass) -> usize {
    match c {
        AdClass::FastPair => 0,
        AdClass::IBeacon => 1,
        AdClass::Proximity => 2,
        AdClass::Generic => 3,
    }
}

pub const ALL_CLASSES: [AdClass; CLASSES] = [
    AdClass::FastPair,
    AdClass::IBeacon,
    AdClass::Proximity,
    AdClass::Generic,
];
