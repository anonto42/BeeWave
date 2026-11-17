use super::*;

fn obs(rssi: i8) -> Observation {
    Observation {
        bssid: [1; 6],
        channel: 1,
        rssi,
        ssid: "x".try_into().unwrap(),
    }
}

#[test]
fn dedupes_until_rssi_moves() {
    let mut t = ScanTable::new();
    assert!(t.observe(obs(-50), 0).is_some());
    assert!(t.observe(obs(-52), 0).is_none());
    assert!(t.observe(obs(-60), 0).is_some());
}

#[test]
fn survives_overflow() {
    let mut t = ScanTable::new();
    for i in 0..200u8 {
        let mut o = obs(-50);
        o.bssid = [i, 0, 0, 0, 0, 1];
        assert!(t.observe(o, 0).is_some());
    }
}

/// Build a beacon: 24-byte header, 12 fixed bytes, then the given information elements.
fn beacon(ies: &[u8]) -> heapless::Vec<u8, 128> {
    let mut f: heapless::Vec<u8, 128> = heapless::Vec::new();
    f.extend_from_slice(&[0x80, 0x00, 0, 0]).unwrap(); // frame control, duration
    f.extend_from_slice(&[0xff; 6]).unwrap(); // addr1 (broadcast)
    f.extend_from_slice(&[0xaa; 6]).unwrap(); // addr2 (transmitter)
    f.extend_from_slice(&[1, 2, 3, 4, 5, 6]).unwrap(); // addr3 = BSSID
    f.extend_from_slice(&[0, 0]).unwrap(); // sequence
    f.extend_from_slice(&[0; 12]).unwrap(); // timestamp, interval, capabilities
    f.extend_from_slice(ies).unwrap();
    f
}

#[test]
fn parses_ssid_and_channel_from_beacon() {
    let f = beacon(&[0, 4, b'H', b'o', b'm', b'e', 3, 1, 6]);
    let o = parse_beacon(&f, -55, 1).unwrap();
    assert_eq!(o.bssid, [1, 2, 3, 4, 5, 6]);
    assert_eq!(o.ssid.as_str(), "Home");
    assert_eq!(o.channel, 6); // DS parameter set wins over the rx channel
    assert_eq!(o.rssi, -55);
}

#[test]
fn hidden_ssid_is_empty_and_rx_channel_used() {
    let f = beacon(&[0, 3, 0, 0, 0]);
    let o = parse_beacon(&f, -70, 11).unwrap();
    assert_eq!(o.ssid.as_str(), "");
    assert_eq!(o.channel, 11);
}

#[test]
fn rejects_non_beacons_and_short_frames() {
    let mut f = beacon(&[0, 0]);
    f[0] = 0x40; // probe request
    assert!(parse_beacon(&f, -50, 1).is_none());
    assert!(parse_beacon(&[0x80, 0, 0], -50, 1).is_none());
}

#[test]
fn truncated_element_does_not_panic() {
    let f = beacon(&[0, 20, b'a', b'b']); // claims 20 bytes, only 2 present
    let o = parse_beacon(&f, -50, 1).unwrap();
    assert_eq!(o.ssid.as_str(), "");
}

// ---- owner scan settings ----------------------------------------------------------------------

use beewave_protocol::ScanSettings;

fn settings() -> ScanSettings {
    crate::ports::Settings::default().scan
}

fn named(bssid: u8, ssid: &str, channel: u8, rssi: i8) -> Observation {
    Observation {
        bssid: [bssid; 6],
        channel,
        rssi,
        ssid: ssid.try_into().unwrap(),
    }
}

#[test]
fn the_dedup_threshold_is_configurable() {
    let mut t = ScanTable::new();
    let mut s = settings();
    s.dedup_rssi_db = 2;
    t.configure(&s);
    assert!(t.observe(obs(-50), 0).is_some());
    assert!(t.observe(obs(-52), 1000).is_some(), "2 dB now counts");
    assert!(t.observe(obs(-53), 2000).is_none(), "1 dB still does not");
    s.dedup_rssi_db = 30;
    t.configure(&s);
    assert!(t.observe(obs(-80), 3000).is_none(), "28 dB is below 30");
    assert!(t.observe(obs(-90), 4000).is_some());
}

#[test]
fn a_network_is_re_reported_no_sooner_than_the_interval() {
    let mut t = ScanTable::new();
    let mut s = settings();
    s.scan_interval_ms = 1000;
    t.configure(&s);
    assert!(t.observe(obs(-50), 0).is_some());
    assert!(
        t.observe(obs(-70), 500).is_none(),
        "moved a lot, but too soon"
    );
    assert!(t.observe(obs(-70), 1000).is_some(), "interval passed");
    assert!(
        t.observe(named(2, "new", 1, -50), 1001).is_some(),
        "new networks are never held back"
    );
}

#[test]
fn the_interval_survives_the_clock_wrapping() {
    let mut t = ScanTable::new();
    let mut s = settings();
    s.scan_interval_ms = 1000;
    t.configure(&s);
    assert!(t.observe(obs(-50), u32::MAX - 100).is_some());
    assert!(
        t.observe(obs(-70), 50).is_none(),
        "151 ms later across the wrap"
    );
    assert!(t.observe(obs(-70), 2000).is_some());
}

#[test]
fn the_ssid_filter_matches_a_substring_ignoring_case() {
    let mut t = ScanTable::new();
    let mut s = settings();
    s.ssid_filter = "caf".try_into().unwrap();
    t.configure(&s);
    assert!(t.observe(named(1, "My CAFE", 1, -50), 0).is_some());
    assert!(t.observe(named(2, "Office", 1, -50), 0).is_none());
    assert!(
        t.observe(named(3, "", 1, -50), 0).is_none(),
        "hidden networks have no SSID to match"
    );
    s.ssid_filter.clear();
    t.configure(&s);
    assert!(
        t.observe(named(2, "Office", 1, -50), 0).is_some(),
        "empty filter = everything"
    );
}

#[test]
fn the_channel_mask_limits_reports_and_zero_means_all() {
    let mut t = ScanTable::new();
    let mut s = settings();
    s.channel_filter_mask = 0b100_0000_0001; // channels 1 and 11
    t.configure(&s);
    assert!(t.observe(named(1, "a", 1, -50), 0).is_some());
    assert!(t.observe(named(2, "b", 6, -50), 0).is_none());
    assert!(t.observe(named(3, "c", 11, -50), 0).is_some());
    s.channel_filter_mask = 0;
    t.configure(&s);
    assert!(t.observe(named(2, "b", 6, -50), 0).is_some());
}
