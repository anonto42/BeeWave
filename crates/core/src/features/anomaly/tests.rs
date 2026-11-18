use beewave_protocol::AnomalyKind;

use super::{AnomalyDetector, CAPACITY, COOLDOWN_TICKS, CYCLE_TICKS, LEARN_TICKS};
use crate::features::scan::Observation;

fn beacon(id: u8, ssid: &str, channel: u8, rssi: i8) -> Observation {
    Observation {
        bssid: [id; 6],
        channel,
        rssi,
        ssid: ssid.try_into().unwrap(),
        wps: None,
        security: None,
    }
}

/// A detector that has finished its learning period with one network ("home", bssid 1) known.
fn learned() -> (AnomalyDetector, u32) {
    let mut d = AnomalyDetector::new();
    let mut t = 0;
    while t <= LEARN_TICKS + CYCLE_TICKS {
        assert!(d.observe(&beacon(1, "home", 6, -50), t).is_none());
        t += 1;
    }
    (d, t)
}

fn kind(a: Option<super::Anomaly>) -> Option<AnomalyKind> {
    a.map(|a| a.kind)
}

#[test]
fn nothing_is_reported_while_learning() {
    let mut d = AnomalyDetector::new();
    assert!(d.observe(&beacon(1, "home", 6, -50), 0).is_none());
    assert!(
        d.observe(&beacon(2, "home", 6, -50), 1).is_none(),
        "second AP of the same SSID"
    );
    assert!(
        d.observe(&beacon(1, "home", 11, -50), 2).is_none(),
        "channel change"
    );
}

#[test]
fn a_known_ssid_on_a_new_bssid_is_flagged() {
    let (mut d, t) = learned();
    let a = d.observe(&beacon(2, "home", 6, -60), t).unwrap();
    assert_eq!((a.kind, a.bssid), (AnomalyKind::NewBssidForSsid, [2; 6]));
    // the same BSSID again is no longer new
    assert!(d.observe(&beacon(2, "home", 6, -60), t + 1).is_none());
}

#[test]
fn new_ssids_and_hidden_networks_are_not_flagged() {
    let (mut d, t) = learned();
    assert!(d.observe(&beacon(3, "cafe", 1, -70), t).is_none());
    assert!(d.observe(&beacon(4, "", 1, -70), t).is_none());
    assert!(
        d.observe(&beacon(5, "", 1, -70), t).is_none(),
        "two hidden networks are unrelated"
    );
}

#[test]
fn a_bssid_on_two_channels_within_a_minute_is_flagged() {
    let (mut d, t) = learned();
    let a = d.observe(&beacon(1, "home", 11, -50), t).unwrap();
    assert_eq!(a.kind, AnomalyKind::TwoChannels);
}

#[test]
fn a_channel_change_after_a_long_gap_is_not_two_channels() {
    let (mut d, t) = learned();
    // keep the entry alive in the same cycle structure, then wait well over a minute with it unheard
    let later = t + 200;
    let a = d.observe(&beacon(1, "home", 11, -50), later);
    assert_eq!(
        kind(a),
        Some(AnomalyKind::Returned),
        "long absence reads as 'returned', not two channels"
    );
}

#[test]
fn an_rssi_jump_over_20_db_between_cycles_is_an_outlier() {
    let (mut d, t) = learned();
    let next_cycle = (t / CYCLE_TICKS + 1) * CYCLE_TICKS;
    let a = d.observe(&beacon(1, "home", 6, -20), next_cycle).unwrap();
    assert_eq!((a.kind, a.detail), (AnomalyKind::RssiOutlier, 30));
}

#[test]
fn small_or_same_cycle_rssi_changes_are_ignored() {
    let (mut d, t) = learned();
    let next_cycle = (t / CYCLE_TICKS + 1) * CYCLE_TICKS;
    assert!(
        d.observe(&beacon(1, "home", 6, -35), next_cycle).is_none(),
        "15 dB"
    );
    // a big jump inside the same cycle is not compared (only between cycles)
    assert!(d
        .observe(&beacon(1, "home", 6, -90), next_cycle + 1)
        .is_none());
}

#[test]
fn a_network_back_after_more_than_five_cycles_has_returned() {
    let (mut d, t) = learned();
    let back = t + (6 + 2) * CYCLE_TICKS; // more than 5 cycles of silence
    let a = d.observe(&beacon(1, "home", 6, -50), back).unwrap();
    assert_eq!(a.kind, AnomalyKind::Returned);
    assert!(a.detail > 5, "detail = cycles missed, got {}", a.detail);
}

#[test]
fn a_short_gap_is_not_a_return() {
    let (mut d, t) = learned();
    let back = t + 3 * CYCLE_TICKS;
    assert!(d.observe(&beacon(1, "home", 6, -50), back).is_none());
}

#[test]
fn the_same_finding_respects_the_cooldown() {
    let (mut d, t) = learned();
    assert!(d.observe(&beacon(1, "home", 11, -50), t).is_some());
    assert!(
        d.observe(&beacon(1, "home", 6, -50), t + 1).is_none(),
        "flip back inside the cooldown"
    );
    assert!(d
        .observe(&beacon(1, "home", 11, -50), t + COOLDOWN_TICKS / 2)
        .is_none());
    assert!(
        d.observe(&beacon(1, "home", 6, -50), t + COOLDOWN_TICKS + 5)
            .is_some(),
        "reported again once the cooldown ended"
    );
}

#[test]
fn the_table_is_bounded_and_drops_the_least_recently_seen() {
    let mut d = AnomalyDetector::new();
    for i in 0..(CAPACITY as u8 + 10) {
        d.observe(&beacon(i, "x", 1, -50), u32::from(i));
    }
    // the oldest were dropped, so bssid 0 reads as brand new (hidden-free, learning over by now)
    let t = LEARN_TICKS + 1000;
    assert!(
        d.observe(&beacon(0, "x", 1, -50), t).is_some(),
        "0 was evicted and 'x' is a known SSID"
    );
}
