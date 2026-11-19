use super::*;

fn obs(rssi: i8) -> Observation {
    Observation {
        bssid: [1; 6],
        channel: 1,
        rssi,
        ssid: "x".try_into().unwrap(),
        wps: None,
        security: None,
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
fn hidden_network_decloaked_bypasses_dedup() {
    let mut t = ScanTable::new();
    let hidden = Observation {
        bssid: [0xaa; 6],
        channel: 6,
        rssi: -50,
        ssid: "".try_into().unwrap(),
        wps: None,
        security: None,
    };
    // 1. Initial observation of hidden AP emits ScanResult
    let res = t.observe(hidden.clone(), 0).expect("first hidden scan");
    assert!(matches!(res, beewave_protocol::TelemetryMessage::ScanResult { ssid, .. } if ssid.is_empty()));

    // 2. Immediate duplicate hidden beacon at identical RSSI is suppressed
    assert!(t.observe(hidden.clone(), 0).is_none());

    // 3. Probe response arrives with cleartext SSID at identical RSSI: MUST emit DecloakedResult
    let probe_resp = Observation {
        bssid: [0xaa; 6],
        channel: 6,
        rssi: -50,
        ssid: "SecretCorp".try_into().unwrap(),
        wps: None,
        security: None,
    };
    let decloaked = t.observe(probe_resp, 10).expect("decloaked report");
    assert_eq!(
        decloaked,
        beewave_protocol::TelemetryMessage::DecloakedResult {
            bssid: [0xaa; 6],
            ssid: "SecretCorp".try_into().unwrap(),
            channel: 6,
            rssi: -50,
        }
    );

    // 4. Subsequent empty beacon at same RSSI is suppressed and does not revert to hidden
    assert!(t.observe(hidden, 20).is_none());
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
        wps: None,
        security: None,
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

#[test]
fn parses_wps_element_and_attributes() {
    let bssid = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
    let mut body = [0u8; 128];
    // OUI
    body[0..4].copy_from_slice(&[0x00, 0x50, 0xF2, 0x04]);
    let mut o = 4;

    // Version 2.0 (0x104A)
    body[o..o + 2].copy_from_slice(&0x104Au16.to_be_bytes());
    body[o + 2..o + 4].copy_from_slice(&1u16.to_be_bytes());
    body[o + 4] = 0x20;
    o += 5;

    // State 2 (Configured)
    body[o..o + 2].copy_from_slice(&0x1044u16.to_be_bytes());
    body[o + 2..o + 4].copy_from_slice(&1u16.to_be_bytes());
    body[o + 4] = 0x02;
    o += 5;

    // Locked = 1
    body[o..o + 2].copy_from_slice(&0x1057u16.to_be_bytes());
    body[o + 2..o + 4].copy_from_slice(&1u16.to_be_bytes());
    body[o + 4] = 0x01;
    o += 5;

    // Device Name = "RouterAX"
    let dev = b"RouterAX";
    body[o..o + 2].copy_from_slice(&0x1011u16.to_be_bytes());
    body[o + 2..o + 4].copy_from_slice(&(dev.len() as u16).to_be_bytes());
    body[o + 4..o + 4 + dev.len()].copy_from_slice(dev);
    o += 4 + dev.len();

    let info = parse_wps_element(bssid, &body[..o]).expect("should parse");
    assert_eq!(info.bssid, bssid);
    assert_eq!(info.version, 0x20);
    assert_eq!(info.state, 2);
    assert!(info.locked);
    assert!(!info.selected_registrar);
    assert_eq!(info.device_name.as_str(), "RouterAX");
}

#[test]
fn wps_tracker_suppresses_unchanged_attributes() {
    let mut tracker = WpsTracker::new();
    let bssid = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
    let mut info = WpsInfo {
        bssid,
        version: 0x20,
        state: 2,
        locked: true,
        selected_registrar: false,
        device_name: "MyAP".try_into().unwrap(),
        manufacturer: "Vendor".try_into().unwrap(),
        model_name: "Model1".try_into().unwrap(),
    };

    // First observation: emitted
    assert!(tracker.observe(&info));

    // Identical observation: suppressed
    assert!(!tracker.observe(&info));

    // Status change (unlocked): emitted
    info.locked = false;
    assert!(tracker.observe(&info));
}

#[test]
fn beacon_with_wps_element_extracts_wps_info() {
    let mut frame = [0u8; 128];
    frame[0] = 0x80; // Beacon
    frame[16..22].copy_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);

    // Fixed params = 12 bytes at 24..36
    // IE_SSID at 36: tag 0, len 4, "WiFi"
    frame[36] = 0;
    frame[37] = 4;
    frame[38..42].copy_from_slice(b"WiFi");

    // IE_DS_PARAMS at 42: tag 3, len 1, channel 6
    frame[42] = 3;
    frame[43] = 1;
    frame[44] = 6;

    // IE_VENDOR_SPECIFIC at 45: tag 221
    frame[45] = 221;
    frame[46] = 9; // len = 4 OUI + 5 version attr
    frame[47..51].copy_from_slice(&[0x00, 0x50, 0xF2, 0x04]);
    frame[51..53].copy_from_slice(&0x104Au16.to_be_bytes());
    frame[53..55].copy_from_slice(&1u16.to_be_bytes());
    frame[55] = 0x20;

    let obs = parse_beacon(&frame[..56], -60, 1).expect("should parse");
    assert_eq!(obs.ssid.as_str(), "WiFi");
    assert_eq!(obs.channel, 6);
    let wps = obs.wps.expect("should have wps info");
    assert_eq!(wps.version, 0x20);
}

fn make_rsn_body(akm_types: &[u8], mfpc: bool, mfpr: bool) -> heapless::Vec<u8, 64> {
    let mut v = heapless::Vec::<u8, 64>::new();
    v.extend_from_slice(&1u16.to_le_bytes()).unwrap();
    v.extend_from_slice(&[0x00, 0x0F, 0xAC, 0x04]).unwrap();
    v.extend_from_slice(&1u16.to_le_bytes()).unwrap();
    v.extend_from_slice(&[0x00, 0x0F, 0xAC, 0x04]).unwrap();
    v.extend_from_slice(&(akm_types.len() as u16).to_le_bytes()).unwrap();
    for &t in akm_types {
        v.extend_from_slice(&[0x00, 0x0F, 0xAC, t]).unwrap();
    }
    let mut caps = 0u16;
    if mfpc {
        caps |= 0x0040;
    }
    if mfpr {
        caps |= 0x0080;
    }
    v.extend_from_slice(&caps.to_le_bytes()).unwrap();
    v
}

#[test]
fn test_rsn_element_parsing() {
    use beewave_protocol::WifiPmfMode;

    // WPA2 PSK, no PMF
    let body_psk = make_rsn_body(&[2], false, false);
    let (dot1x, psk, sae, owe, pmf) = parse_rsn_element(&body_psk);
    assert!(!dot1x && psk && !sae && !owe && pmf == WifiPmfMode::None);

    // WPA3 SAE, PMF required
    let body_sae = make_rsn_body(&[8], true, true);
    let (dot1x, psk, sae, owe, pmf) = parse_rsn_element(&body_sae);
    assert!(!dot1x && !psk && sae && !owe && pmf == WifiPmfMode::Required);

    // Transition PSK + SAE, PMF capable
    let body_trans = make_rsn_body(&[2, 8], true, false);
    let (dot1x, psk, sae, owe, pmf) = parse_rsn_element(&body_trans);
    assert!(!dot1x && psk && sae && !owe && pmf == WifiPmfMode::Capable);

    // OWE
    let body_owe = make_rsn_body(&[18], true, true);
    let (dot1x, psk, sae, owe, pmf) = parse_rsn_element(&body_owe);
    assert!(!dot1x && !psk && !sae && owe && pmf == WifiPmfMode::Required);
}

#[test]
fn test_security_classification() {
    use beewave_protocol::{WifiPmfMode, WifiSecurityProtocol};

    // Open network (no privacy, no RSN, no WPA1)
    let (proto, pmf) = classify_security(false, None, false);
    assert_eq!(proto, WifiSecurityProtocol::Open);
    assert_eq!(pmf, WifiPmfMode::None);

    // WEP network (privacy bit set, no RSN, no WPA1)
    let (proto, pmf) = classify_security(true, None, false);
    assert_eq!(proto, WifiSecurityProtocol::Wep);
    assert_eq!(pmf, WifiPmfMode::None);

    // WPA1 network (vendor specific element present)
    let (proto, pmf) = classify_security(true, None, true);
    assert_eq!(proto, WifiSecurityProtocol::Wpa);
    assert_eq!(pmf, WifiPmfMode::None);

    // WPA2 PSK
    let (proto, pmf) = classify_security(true, Some((false, true, false, false, WifiPmfMode::Capable)), false);
    assert_eq!(proto, WifiSecurityProtocol::Wpa2);
    assert_eq!(pmf, WifiPmfMode::Capable);

    // WPA3 SAE
    let (proto, pmf) = classify_security(true, Some((false, false, true, false, WifiPmfMode::None)), false);
    assert_eq!(proto, WifiSecurityProtocol::Wpa3);
    assert_eq!(pmf, WifiPmfMode::Required); // Defaulted to required for SAE

    // WPA2/WPA3 Transition
    let (proto, pmf) = classify_security(true, Some((false, true, true, false, WifiPmfMode::None)), false);
    assert_eq!(proto, WifiSecurityProtocol::Wpa2Wpa3);
    assert_eq!(pmf, WifiPmfMode::Capable);

    // Enterprise 802.1X
    let (proto, pmf) = classify_security(true, Some((true, false, false, false, WifiPmfMode::None)), false);
    assert_eq!(proto, WifiSecurityProtocol::Enterprise);
    assert_eq!(pmf, WifiPmfMode::None);
}

#[test]
fn test_security_tracker_deduplication() {
    use beewave_protocol::{WifiPmfMode, WifiSecurityProtocol};

    let mut tracker = SecurityTracker::new();
    let mut info = SecurityInfo {
        bssid: [1, 2, 3, 4, 5, 6],
        protocol: WifiSecurityProtocol::Wpa2,
        pmf: WifiPmfMode::None,
    };

    assert!(tracker.observe(&info)); // first time seen -> true
    assert!(!tracker.observe(&info)); // unchanged -> false

    // Posture changed (upgrade to PMF Capable)
    info.pmf = WifiPmfMode::Capable;
    assert!(tracker.observe(&info)); // changed -> true
    assert!(!tracker.observe(&info)); // unchanged -> false
}

#[test]
fn beacon_with_rsn_element_extracts_security_info() {
    use beewave_protocol::{WifiPmfMode, WifiSecurityProtocol};

    let mut frame = [0u8; 128];
    frame[0] = 0x80; // Beacon
    frame[16..22].copy_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);

    // Capability info: privacy bit set (0x0010)
    frame[34] = 0x10;
    frame[35] = 0x00;

    // IE_SSID at 36: tag 0, len 4, "Wpa3"
    frame[36] = 0;
    frame[37] = 4;
    frame[38..42].copy_from_slice(b"Wpa3");

    // IE_RSN at 42: tag 48
    let rsn_body = make_rsn_body(&[8], true, true);
    frame[42] = 48;
    frame[43] = rsn_body.len() as u8;
    frame[44..44 + rsn_body.len()].copy_from_slice(&rsn_body);

    let total_len = 44 + rsn_body.len();
    let obs = parse_beacon(&frame[..total_len], -55, 6).expect("should parse");
    assert_eq!(obs.ssid.as_str(), "Wpa3");
    let sec = obs.security.expect("should have security info");
    assert_eq!(sec.protocol, WifiSecurityProtocol::Wpa3);
    assert_eq!(sec.pmf, WifiPmfMode::Required);
}

#[test]
fn beacon_flood_detector_triggers_on_high_rate_and_unique_bssids() {
    let mut detector = BeaconFloodDetector::new();
    assert_eq!(detector.threshold(), 25);

    // 20 beacons from 20 BSSIDs: under threshold of 25
    for i in 0..20u8 {
        detector.observe(&[0, 0, 0, 0, 0, i]);
    }
    assert!(detector.tick().is_none());

    // 30 beacons from only 3 BSSIDs: not enough unique BSSIDs (min 8)
    for _ in 0..10 {
        detector.observe(&[0, 0, 0, 0, 0, 1]);
        detector.observe(&[0, 0, 0, 0, 0, 2]);
        detector.observe(&[0, 0, 0, 0, 0, 3]);
    }
    assert!(detector.tick().is_none());

    // 30 beacons from 15 unique BSSIDs: triggers flood alert!
    for i in 0..15u8 {
        detector.observe(&[0, 0, 0, 0, 1, i]);
        detector.observe(&[0, 0, 0, 0, 1, i]);
    }
    let alert = detector.tick().expect("should trigger flood alert");
    match alert {
        beewave_protocol::TelemetryMessage::BeaconFloodAlert { rate, unique_bssids } => {
            assert_eq!(rate, 30);
            assert_eq!(unique_bssids, 15);
        }
        _ => panic!("wrong message"),
    }

    // Cooldown active: consecutive tick with high rate does not re-alert immediately
    for i in 0..15u8 {
        detector.observe(&[0, 0, 0, 0, 2, i]);
        detector.observe(&[0, 0, 0, 0, 2, i]);
    }
    assert!(detector.tick().is_none());
}
