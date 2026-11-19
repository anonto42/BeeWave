use beewave_protocol::AdClass;

use super::flood::{BleFloodDetector, BLE_FLOOD_COOLDOWN_TICKS, DEFAULT_BLE_FLOOD_THRESHOLD};
use super::{classify, BleMonitor, STATS_EVERY_SECS, WARMUP_SECS};

// ---- classifier -------------------------------------------------------------------------------

/// `[len][type][data]` structures glued together.
fn ad(structures: &[(u8, &[u8])]) -> std::vec::Vec<u8> {
    let mut out = std::vec::Vec::new();
    for (ty, data) in structures {
        out.push(data.len() as u8 + 1);
        out.push(*ty);
        out.extend_from_slice(data);
    }
    out
}

#[test]
fn recognises_fast_pair_service_data() {
    let p = ad(&[(0x01, &[0x06]), (0x16, &[0x2C, 0xFE, 0x00, 0xAA, 0xBB])]);
    assert_eq!(classify(&p), AdClass::FastPair);
}

#[test]
fn other_service_data_is_generic() {
    assert_eq!(
        classify(&ad(&[(0x16, &[0x0F, 0x18, 0x64])])),
        AdClass::Generic
    );
}

#[test]
fn recognises_ibeacon() {
    let mut mfg = std::vec![0x4C, 0x00, 0x02, 0x15];
    mfg.extend_from_slice(&[0u8; 21]);
    assert_eq!(classify(&ad(&[(0xFF, &mfg)])), AdClass::IBeacon);
}

#[test]
fn recognises_airpods_style_proximity_pairing() {
    let mfg = [0x4C, 0x00, 0x07, 0x19, 0x01, 0x02, 0x20];
    assert_eq!(classify(&ad(&[(0xFF, &mfg)])), AdClass::Proximity);
}

#[test]
fn other_apple_and_other_vendor_data_are_generic() {
    assert_eq!(
        classify(&ad(&[(0xFF, &[0x4C, 0x00, 0x10, 0x05, 1])])),
        AdClass::Generic
    );
    assert_eq!(
        classify(&ad(&[(0xFF, &[0x06, 0x00, 0x02, 0x15, 1])])),
        AdClass::Generic
    );
}

#[test]
fn malformed_payloads_never_panic() {
    assert_eq!(classify(&[]), AdClass::Generic);
    assert_eq!(classify(&[0]), AdClass::Generic);
    assert_eq!(
        classify(&[9, 0xFF, 0x4C]),
        AdClass::Generic,
        "length runs past the end"
    );
    assert_eq!(
        classify(&[2, 0x16, 0x2C]),
        AdClass::Generic,
        "truncated service data"
    );
    // a good structure before a broken one still classifies
    let mut p = ad(&[(0x16, &[0x2C, 0xFE])]);
    p.extend_from_slice(&[0x20, 0xFF]);
    assert_eq!(classify(&p), AdClass::FastPair);
}

// ---- counting and spikes ----------------------------------------------------------------------

fn run(m: &mut BleMonitor, secs: u32, per_sec: u16, class: AdClass) {
    for _ in 0..secs {
        for _ in 0..per_sec {
            m.observe(class);
        }
        m.tick();
    }
}

#[test]
fn counts_cover_the_last_minute_only() {
    let mut m = BleMonitor::new();
    run(&mut m, 30, 2, AdClass::IBeacon);
    assert_eq!(m.counts()[1], 60);
    run(&mut m, 60, 0, AdClass::IBeacon);
    assert_eq!(m.counts()[1], 0, "everything has aged out");
}

#[test]
fn publishes_stats_every_ten_seconds() {
    let mut m = BleMonitor::new();
    let mut stats = 0;
    for _ in 0..30 {
        m.observe(AdClass::Generic);
        if m.tick().stats.is_some() {
            stats += 1;
        }
    }
    assert_eq!(stats, 30 / STATS_EVERY_SECS);
}

#[test]
fn a_flood_after_warm_up_raises_a_spike() {
    let mut m = BleMonitor::new();
    run(&mut m, WARMUP_SECS + 20, 1, AdClass::Proximity); // steady 60 per minute
    let mut spike = None;
    for _ in 0..10 {
        for _ in 0..50 {
            m.observe(AdClass::Proximity);
        }
        spike = spike.or(m.tick().spike);
    }
    let s = spike.expect("flood must raise a spike");
    assert_eq!(s.class, AdClass::Proximity);
    assert!(s.count > s.baseline * 3, "{s:?}");
}

#[test]
fn nothing_spikes_during_warm_up_or_steady_traffic() {
    let mut m = BleMonitor::new();
    for _ in 0..WARMUP_SECS {
        for _ in 0..500 {
            m.observe(AdClass::FastPair); // huge, but the baseline is still forming
        }
        assert!(m.tick().spike.is_none());
    }
    let mut steady = BleMonitor::new();
    for _ in 0..(WARMUP_SECS * 3) {
        steady.observe(AdClass::Generic);
        assert!(steady.tick().spike.is_none());
    }
}

#[test]
fn a_spike_is_reported_once_per_cooldown() {
    let mut m = BleMonitor::new();
    run(&mut m, WARMUP_SECS + 10, 1, AdClass::FastPair);
    let mut reports = 0;
    for _ in 0..20 {
        for _ in 0..100 {
            m.observe(AdClass::FastPair);
        }
        if m.tick().spike.is_some() {
            reports += 1;
        }
    }
    assert_eq!(reports, 1);
}

#[test]
fn a_higher_factor_needs_a_bigger_flood() {
    let mut m = BleMonitor::new();
    m.set_factor(20);
    run(&mut m, WARMUP_SECS + 10, 1, AdClass::Generic);
    let mut spiked = false;
    for _ in 0..5 {
        for _ in 0..10 {
            m.observe(AdClass::Generic);
        }
        spiked |= m.tick().spike.is_some();
    }
    assert!(!spiked, "10x is not 20x");
}

#[test]
fn reset_forgets_history_but_keeps_the_factor() {
    let mut m = BleMonitor::new();
    m.set_factor(7);
    run(&mut m, 20, 3, AdClass::Generic);
    m.reset();
    assert_eq!(m.counts(), [0; 4]);
    assert_eq!(m.factor(), 7);
}

#[test]
fn ble_flood_detector_triggers_on_high_packet_rate_and_identifies_dominant_class() {
    let mut d = BleFloodDetector::new();
    assert_eq!(d.threshold(), DEFAULT_BLE_FLOOD_THRESHOLD);

    // 1. Normal ambient packets below threshold (e.g. 5 generic, 2 proximity) -> no alert
    for _ in 0..5 {
        d.observe(AdClass::Generic);
    }
    for _ in 0..2 {
        d.observe(AdClass::Proximity);
    }
    assert!(d.tick().is_none());

    // 2. High-rate flood of Apple proximity popups (e.g. 40 packets)
    for _ in 0..35 {
        d.observe(AdClass::Proximity);
    }
    for _ in 0..5 {
        d.observe(AdClass::Generic);
    }
    let alert = d.tick().expect("flood alert triggered");
    assert_eq!(
        alert,
        beewave_protocol::TelemetryMessage::BleFloodAlert {
            rate: 40,
            class: AdClass::Proximity,
        }
    );

    // 3. Consecutive seconds during flood obey cooldown
    for _ in 0..50 {
        d.observe(AdClass::Proximity);
    }
    assert!(d.tick().is_none(), "in cooldown");

    // Advance through the remaining cooldown ticks
    for _ in 1..BLE_FLOOD_COOLDOWN_TICKS {
        d.tick();
    }

    // Now cooldown has elapsed: subsequent burst alerts again
    for _ in 0..45 {
        d.observe(AdClass::FastPair);
    }
    let alert2 = d.tick().expect("alert after cooldown");
    assert_eq!(
        alert2,
        beewave_protocol::TelemetryMessage::BleFloodAlert {
            rate: 45,
            class: AdClass::FastPair,
        }
    );
}
