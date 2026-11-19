use heapless::Vec;

use super::{parse_probe_request, ProbeFloodDetector, ProbeObservation, ProbeTracker};

const CLIENT: [u8; 6] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
const BROADCAST: [u8; 6] = [0xFF; 6];

/// Builds a minimal 802.11 Probe Request frame (subtype 0x40) with given client MAC,
/// optional SSID, and optional channel DS Parameter Set element.
fn probe_frame(client: [u8; 6], ssid: Option<&str>, ds_channel: Option<u8>) -> Vec<u8, 128> {
    let mut f: Vec<u8, 128> = Vec::new();
    // Frame Control: subtype 0x40 (Probe Request), version 0
    f.extend_from_slice(&[0x40, 0x00]).unwrap();
    // Duration
    f.extend_from_slice(&[0x00, 0x00]).unwrap();
    // Addr1: Destination (Broadcast)
    f.extend_from_slice(&BROADCAST).unwrap();
    // Addr2: Source (Client)
    f.extend_from_slice(&client).unwrap();
    // Addr3: BSSID (Broadcast)
    f.extend_from_slice(&BROADCAST).unwrap();
    // Sequence control
    f.extend_from_slice(&[0x00, 0x00]).unwrap();

    // Information Element 0: SSID
    let ssid_bytes = ssid.unwrap_or("").as_bytes();
    f.push(0).unwrap(); // Element ID: SSID
    f.push(ssid_bytes.len() as u8).unwrap();
    f.extend_from_slice(ssid_bytes).unwrap();

    // Optional Information Element 3: DS Parameter Set (Channel)
    if let Some(ch) = ds_channel {
        f.push(3).unwrap();
        f.push(1).unwrap();
        f.push(ch).unwrap();
    }

    f
}

#[test]
fn parses_directed_probe_request() {
    let frame = probe_frame(CLIENT, Some("CoffeeShop"), Some(6));
    let obs = parse_probe_request(&frame, -60, 1).expect("valid probe request");
    assert_eq!(obs.client, CLIENT);
    assert_eq!(obs.ssid.as_str(), "CoffeeShop");
    assert_eq!(obs.channel, 6);
    assert_eq!(obs.rssi, -60);
}

#[test]
fn parses_wildcard_broadcast_probe_request() {
    let frame = probe_frame(CLIENT, None, None);
    let obs = parse_probe_request(&frame, -75, 11).expect("valid wildcard probe");
    assert_eq!(obs.client, CLIENT);
    assert_eq!(obs.ssid.as_str(), "");
    assert_eq!(obs.channel, 11); // fall back to rx_channel
    assert_eq!(obs.rssi, -75);
}

#[test]
fn rejects_non_probe_and_truncated_frames() {
    // Too short
    assert!(parse_probe_request(&[0x40; 10], -50, 1).is_none());

    // Beacon (0x80) instead of probe request (0x40)
    let mut beacon = probe_frame(CLIENT, Some("Test"), None);
    beacon[0] = 0x80;
    assert!(parse_probe_request(&beacon, -50, 1).is_none());

    // QoS Data (0x88)
    let mut data = probe_frame(CLIENT, Some("Test"), None);
    data[0] = 0x88;
    assert!(parse_probe_request(&data, -50, 1).is_none());
}

#[test]
fn tracker_suppresses_duplicate_probes() {
    let mut tracker = ProbeTracker::new();
    let obs = ProbeObservation {
        client: CLIENT,
        channel: 6,
        rssi: -70,
        ssid: heapless::String::try_from("HomeNetwork").unwrap(),
    };

    assert!(tracker.observe(&obs).is_some());
    // Duplicate probe with same RSSI within interval: suppressed
    assert!(tracker.observe(&obs).is_none());
}

#[test]
fn tracker_emits_on_significant_rssi_delta() {
    let mut tracker = ProbeTracker::new();
    let obs1 = ProbeObservation {
        client: CLIENT,
        channel: 6,
        rssi: -70,
        ssid: heapless::String::try_from("HomeNetwork").unwrap(),
    };
    assert!(tracker.observe(&obs1).is_some());

    // RSSI jumps from -70 to -60 (delta 10 dB >= 6 dB threshold): emits
    let mut obs2 = obs1.clone();
    obs2.rssi = -60;
    assert!(tracker.observe(&obs2).is_some());
}

#[test]
fn tracker_is_bounded_and_evicts_oldest() {
    let mut tracker = ProbeTracker::new();
    for i in 0..(super::PROBE_CAPACITY as u8 + 5) {
        let obs = ProbeObservation {
            client: [i, 2, 3, 4, 5, 6],
            channel: 1,
            rssi: -50,
            ssid: heapless::String::new(),
        };
        assert!(tracker.observe(&obs).is_some());
        tracker.tick();
    }
}

#[test]
fn probe_flood_detector_triggers_on_high_rate_and_unique_clients() {
    let mut detector = ProbeFloodDetector::new();
    assert_eq!(detector.threshold(), super::DEFAULT_PROBE_FLOOD_THRESHOLD);

    // 1. Normal ambient traffic: 5 probes from 3 clients -> no alert
    for i in 0..5 {
        detector.observe(&[0x11, 0x22, 0x33, 0x44, 0x55, i % 3]);
    }
    assert!(detector.tick().is_none());

    // 2. High-rate burst from a single client (aggressive scanning) -> no flood alert
    for _ in 0..35 {
        detector.observe(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x01]);
    }
    assert!(detector.tick().is_none());

    // 3. Flood attack: 35 probes from 12 distinct spoofed clients -> triggers alert!
    for i in 0..35 {
        detector.observe(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, (i % 12) as u8]);
    }
    let alert = detector.tick().expect("should trigger probe flood alert");
    match alert {
        beewave_protocol::TelemetryMessage::ProbeFloodAlert {
            rate,
            unique_clients,
        } => {
            assert_eq!(rate, 35);
            assert_eq!(unique_clients, 12);
        }
        other => panic!("unexpected telemetry: {:?}", other),
    }

    // 4. Cooldown suppresses consecutive alerts
    for i in 0..35 {
        detector.observe(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, (i % 12) as u8]);
    }
    assert!(detector.tick().is_none());
}
