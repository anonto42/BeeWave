use super::{DeauthDetector, DEFAULT_THRESHOLD, MAX_ALERTS};

const A: [u8; 6] = [1; 6];
const B: [u8; 6] = [2; 6];

#[test]
fn defaults_to_ten_per_second() {
    assert_eq!(DeauthDetector::new().threshold(), DEFAULT_THRESHOLD);
    assert_eq!(DEFAULT_THRESHOLD, 10);
}

#[test]
fn alerts_at_the_threshold_but_not_below() {
    let d = DeauthDetector::new();
    assert!(d.evaluate(&[(A, 9)]).is_empty());
    let hits = d.evaluate(&[(A, 10), (B, 3)]);
    assert_eq!(hits.len(), 1);
    assert_eq!((hits[0].bssid, hits[0].count), (A, 10));
}

#[test]
fn a_zero_threshold_turns_it_off() {
    let mut d = DeauthDetector::new();
    d.set_threshold(0);
    assert!(d.evaluate(&[(A, 60_000)]).is_empty());
}

#[test]
fn the_threshold_is_configurable() {
    let mut d = DeauthDetector::new();
    d.set_threshold(3);
    assert_eq!(d.evaluate(&[(A, 3)]).len(), 1);
}

#[test]
fn reports_the_strongest_first_and_is_bounded() {
    let d = DeauthDetector::new();
    let counts: std::vec::Vec<_> = (1..=8u8).map(|i| ([i; 6], 10 + u16::from(i))).collect();
    let hits = d.evaluate(&counts);
    assert_eq!(hits.len(), MAX_ALERTS);
    let got: std::vec::Vec<u16> = hits.iter().map(|h| h.count).collect();
    assert_eq!(got, [18, 17, 16, 15]);
}
