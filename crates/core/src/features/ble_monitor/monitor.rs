use beewave_protocol::AdClass;

use super::types::{
    class_index, BleCounts, BleSpikeHit, BleTick, ALL_CLASSES, CLASSES, MIN_SPIKE_DELTA,
    SPIKE_COOLDOWN_SECS, STATS_EVERY_SECS, WARMUP_SECS, WINDOW_SECS,
};

/// Baseline is kept in 1/16 advertisement units so a slow average does not round to zero.
const FP: u32 = 16;
/// How fast the baseline follows the count: 1/16 of the difference per second.
const BASELINE_SHIFT: u32 = 4;
pub const DEFAULT_FACTOR: u8 = 3;

/// Per-class counts over the last `WINDOW_SECS` seconds (one bucket per second) and spike detection
/// against a slowly learned baseline. Fixed size: 480 bytes of counters.
pub struct BleMonitor {
    buckets: [[u16; WINDOW_SECS]; CLASSES],
    head: usize,
    secs: u32,
    baseline: [u32; CLASSES],
    quiet_until: [u32; CLASSES],
    factor: u8,
}

impl BleMonitor {
    pub const fn new() -> Self {
        Self {
            buckets: [[0; WINDOW_SECS]; CLASSES],
            head: 0,
            secs: 0,
            baseline: [0; CLASSES],
            quiet_until: [0; CLASSES],
            factor: DEFAULT_FACTOR,
        }
    }

    pub fn factor(&self) -> u8 {
        self.factor
    }

    pub fn set_factor(&mut self, factor: u8) {
        self.factor = factor;
    }

    /// Forget everything (called when the monitor is switched on).
    pub fn reset(&mut self) {
        let factor = self.factor;
        *self = Self::new();
        self.factor = factor;
    }

    /// Count one received advertisement in the current second.
    pub fn observe(&mut self, class: AdClass) {
        let slot = &mut self.buckets[class_index(class)][self.head];
        *slot = slot.saturating_add(1);
    }

    /// Advance one second. Publishes the minute counts every `STATS_EVERY_SECS` and at most one spike.
    pub fn tick(&mut self) -> BleTick {
        self.head = (self.head + 1) % WINDOW_SECS;
        for class in &mut self.buckets {
            class[self.head] = 0;
        }
        self.secs += 1;

        let counts = self.counts();
        let spike = self.update_baseline(&counts);
        BleTick {
            stats: (self.secs % STATS_EVERY_SECS == 0).then_some(counts),
            spike,
        }
    }

    /// Advertisements per class over the last minute.
    pub fn counts(&self) -> BleCounts {
        let mut out = [0u16; CLASSES];
        for (o, class) in out.iter_mut().zip(&self.buckets) {
            *o = class.iter().fold(0u16, |a, &b| a.saturating_add(b));
        }
        out
    }

    fn update_baseline(&mut self, counts: &BleCounts) -> Option<BleSpikeHit> {
        let mut found: Option<BleSpikeHit> = None;
        for (i, &count) in counts.iter().enumerate() {
            let base = self.baseline[i] / FP;
            let limit = (base.saturating_mul(u32::from(self.factor)))
                .max(base + u32::from(MIN_SPIKE_DELTA));
            let spiking = self.secs > WARMUP_SECS && u32::from(count) > limit;
            if spiking && self.secs >= self.quiet_until[i] {
                self.quiet_until[i] = self.secs + SPIKE_COOLDOWN_SECS;
                let hit = BleSpikeHit {
                    class: ALL_CLASSES[i],
                    count,
                    baseline: base.min(u32::from(u16::MAX)) as u16,
                };
                if found.is_none_or(|f| hit.count > f.count) {
                    found = Some(hit);
                }
            }
            // learn from normal traffic only, so a flood does not become the new normal
            if !spiking {
                let target = u32::from(count) * FP;
                let b = &mut self.baseline[i];
                *b = if self.secs <= WARMUP_SECS {
                    // warm-up: track the count so the baseline is meaningful when it ends
                    target
                } else if target >= *b {
                    *b + ((target - *b) >> BASELINE_SHIFT)
                } else {
                    *b - ((*b - target) >> BASELINE_SHIFT)
                };
            }
        }
        found
    }
}

impl Default for BleMonitor {
    fn default() -> Self {
        Self::new()
    }
}
