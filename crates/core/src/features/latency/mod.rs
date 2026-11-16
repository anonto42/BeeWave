//! Latency statistics for the coexistence measurements: a fixed-size histogram with percentile
//! queries. Pure data structure (no clock): the firmware measures, this aggregates.
mod histogram;

pub use histogram::{LatencyHistogram, LatencySummary, BUCKET_BOUNDS_US};

#[cfg(test)]
mod tests;
