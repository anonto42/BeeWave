/// Upper edge of each bucket in microseconds (a sample equal to an edge falls in that bucket).
/// One extra overflow bucket catches everything above the last edge.
pub const BUCKET_BOUNDS_US: [u32; 10] = [
    500, 1_000, 2_000, 5_000, 10_000, 20_000, 50_000, 100_000, 200_000, 500_000,
];
const BUCKETS: usize = BUCKET_BOUNDS_US.len() + 1;

/// Counts samples per bucket. Fixed size, no allocation.
#[derive(Clone, Debug)]
pub struct LatencyHistogram {
    pub(super) counts: [u32; BUCKETS],
    pub(super) total: u32,
    max_us: u32,
}

/// The numbers worth logging. Percentiles are bucket upper edges (an upper bound, not exact).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LatencySummary {
    pub count: u32,
    pub p50_us: u32,
    pub p99_us: u32,
    pub max_us: u32,
}

impl Default for LatencyHistogram {
    fn default() -> Self {
        Self::new()
    }
}

impl LatencyHistogram {
    pub const fn new() -> Self {
        Self {
            counts: [0; BUCKETS],
            total: 0,
            max_us: 0,
        }
    }

    pub fn record(&mut self, micros: u32) {
        let bucket = BUCKET_BOUNDS_US
            .iter()
            .position(|&edge| micros <= edge)
            .unwrap_or(BUCKETS - 1);
        self.counts[bucket] = self.counts[bucket].saturating_add(1);
        self.total = self.total.saturating_add(1);
        self.max_us = self.max_us.max(micros);
    }

    pub fn count(&self) -> u32 {
        self.total
    }

    pub fn max_us(&self) -> u32 {
        self.max_us
    }

    /// Upper bound of the bucket holding the `percent`-th percentile (`percent` 1..=100).
    /// `None` when there are no samples. Samples above the last edge report the true maximum.
    pub fn percentile_us(&self, percent: u8) -> Option<u32> {
        if self.total == 0 {
            return None;
        }
        let percent = u64::from(percent.clamp(1, 100));
        // Smallest rank (1-based) that covers `percent` of the samples.
        let rank = (u64::from(self.total) * percent).div_ceil(100).max(1);
        let mut seen = 0u64;
        for (i, &n) in self.counts.iter().enumerate() {
            seen += u64::from(n);
            if seen >= rank {
                return Some(BUCKET_BOUNDS_US.get(i).copied().unwrap_or(self.max_us));
            }
        }
        Some(self.max_us)
    }

    pub fn summary(&self) -> Option<LatencySummary> {
        Some(LatencySummary {
            count: self.total,
            p50_us: self.percentile_us(50)?,
            p99_us: self.percentile_us(99)?,
            max_us: self.max_us,
        })
    }

    /// True when at least `percent` of samples are at or under `budget_us` (conservative: uses
    /// bucket edges, so choose a budget that is a bucket edge such as 50_000).
    pub fn within(&self, percent: u8, budget_us: u32) -> bool {
        self.percentile_us(percent).is_some_and(|p| p <= budget_us)
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
