use super::*;

fn filled(samples: &[(u32, u32)]) -> LatencyHistogram {
    let mut h = LatencyHistogram::new();
    for &(us, times) in samples {
        for _ in 0..times {
            h.record(us);
        }
    }
    h
}

#[test]
fn empty_histogram_has_no_percentiles() {
    let h = LatencyHistogram::new();
    assert_eq!(h.count(), 0);
    assert_eq!(h.percentile_us(50), None);
    assert_eq!(h.summary(), None);
    assert!(!h.within(99, 50_000));
}

#[test]
fn a_sample_equal_to_an_edge_lands_in_that_bucket() {
    let h = filled(&[(1_000, 1)]);
    assert_eq!(h.percentile_us(100), Some(1_000));
    let h = filled(&[(1_001, 1)]);
    assert_eq!(h.percentile_us(100), Some(2_000));
}

#[test]
fn percentiles_follow_the_distribution() {
    // 99 fast samples and 1 slow one.
    let h = filled(&[(800, 99), (80_000, 1)]);
    assert_eq!(h.count(), 100);
    assert_eq!(h.percentile_us(50), Some(1_000));
    assert_eq!(h.percentile_us(99), Some(1_000)); // rank 99 is still a fast sample
    assert_eq!(h.percentile_us(100), Some(100_000)); // the slow one
    assert_eq!(h.max_us(), 80_000);
}

#[test]
fn two_percent_slow_pushes_p99_up() {
    let h = filled(&[(800, 98), (80_000, 2)]);
    assert_eq!(h.percentile_us(99), Some(100_000));
}

#[test]
fn overflow_reports_the_true_maximum() {
    let h = filled(&[(1_000, 5), (900_000, 5)]);
    assert_eq!(h.percentile_us(100), Some(900_000));
    assert_eq!(h.summary().unwrap().max_us, 900_000);
}

#[test]
fn within_checks_the_latency_budget() {
    let good = filled(&[(3_000, 990), (40_000, 10)]);
    assert!(good.within(99, 50_000));
    let bad = filled(&[(3_000, 900), (150_000, 100)]);
    assert!(!bad.within(99, 50_000));
}

#[test]
fn summary_matches_the_parts() {
    let h = filled(&[(400, 50), (4_000, 49), (60_000, 1)]);
    assert_eq!(
        h.summary(),
        Some(LatencySummary {
            count: 100,
            p50_us: 500,
            p99_us: 5_000,
            max_us: 60_000
        })
    );
}

#[test]
fn reset_clears_everything() {
    let mut h = filled(&[(100, 3)]);
    h.reset();
    assert_eq!(h.count(), 0);
    assert_eq!(h.max_us(), 0);
    assert_eq!(h.summary(), None);
}

#[test]
fn counts_saturate_instead_of_wrapping() {
    let mut h = LatencyHistogram::new();
    h.total = u32::MAX;
    h.counts[0] = u32::MAX;
    h.record(1);
    assert_eq!(h.count(), u32::MAX);
}
