use super::parser::parse_data_association;
use super::tracker::AssocTracker;
use super::types::{AssocObservation, ASSOC_CAPACITY};

fn make_data_frame(fc0: u8, fc1: u8, addr1: [u8; 6], addr2: [u8; 6], addr3: [u8; 6]) -> [u8; 24] {
    let mut f = [0u8; 24];
    f[0] = fc0;
    f[1] = fc1;
    f[4..10].copy_from_slice(&addr1);
    f[10..16].copy_from_slice(&addr2);
    f[16..22].copy_from_slice(&addr3);
    f
}

#[test]
fn parses_tods_client_to_ap_data_frame() {
    let bssid = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
    let client = [0x00, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    // fc0 = 0x08 (Data), fc1 = 0x01 (ToDS=1, FromDS=0)
    // In ToDS: addr1 = BSSID (RA), addr2 = Client (TA/SA)
    let frame = make_data_frame(0x08, 0x01, bssid, client, bssid);

    let parsed = parse_data_association(&frame, -60, 6).expect("should parse");
    assert_eq!(parsed.bssid, bssid);
    assert_eq!(parsed.client, client);
    assert_eq!(parsed.channel, 6);
    assert_eq!(parsed.rssi, -60);
}

#[test]
fn parses_fromds_ap_to_client_data_frame() {
    let bssid = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
    let client = [0x00, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    // fc0 = 0x88 (QoS Data), fc1 = 0x02 (ToDS=0, FromDS=1)
    // In FromDS: addr1 = Client (RA/DA), addr2 = BSSID (TA)
    let frame = make_data_frame(0x88, 0x02, client, bssid, bssid);

    let parsed = parse_data_association(&frame, -45, 11).expect("should parse");
    assert_eq!(parsed.bssid, bssid);
    assert_eq!(parsed.client, client);
    assert_eq!(parsed.channel, 11);
    assert_eq!(parsed.rssi, -45);
}

#[test]
fn rejects_ibss_and_wds_frames() {
    let bssid = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
    let client = [0x00, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    // ToDS=0, FromDS=0
    let ibss = make_data_frame(0x08, 0x00, client, bssid, bssid);
    assert!(parse_data_association(&ibss, -50, 1).is_none());

    // ToDS=1, FromDS=1
    let wds = make_data_frame(0x08, 0x03, client, bssid, bssid);
    assert!(parse_data_association(&wds, -50, 1).is_none());
}

#[test]
fn rejects_non_data_and_truncated_frames() {
    let bssid = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
    // Management frame: fc0 = 0x80 (Beacon)
    let beacon = make_data_frame(0x80, 0x00, bssid, bssid, bssid);
    assert!(parse_data_association(&beacon, -50, 1).is_none());

    // Truncated: less than 24 bytes
    let short = [0u8; 20];
    assert!(parse_data_association(&short, -50, 1).is_none());
}

#[test]
fn rejects_multicast_or_all_zero_addresses() {
    let bssid = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
    let broadcast_client = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    let multicast_client = [0x01, 0x00, 0x5E, 0x00, 0x00, 0x01];
    let zero_client = [0x00; 6];

    let bcast_frame = make_data_frame(0x08, 0x01, bssid, broadcast_client, bssid);
    assert!(parse_data_association(&bcast_frame, -50, 1).is_none());

    let mcast_frame = make_data_frame(0x08, 0x01, bssid, multicast_client, bssid);
    assert!(parse_data_association(&mcast_frame, -50, 1).is_none());

    let zero_frame = make_data_frame(0x08, 0x01, bssid, zero_client, bssid);
    assert!(parse_data_association(&zero_frame, -50, 1).is_none());
}

#[test]
fn tracker_suppresses_duplicates_within_window_and_rssi() {
    let mut tracker = AssocTracker::new();
    let obs = AssocObservation {
        bssid: [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
        client: [0x00, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE],
        channel: 6,
        rssi: -60,
    };

    assert!(tracker.observe(&obs).is_some());

    // Same RSSI, only 1 second later: suppressed
    tracker.tick();
    assert!(tracker.observe(&obs).is_none());

    // Small RSSI change (+2 dB), still within 5 seconds: suppressed
    let mut slightly_moved = obs;
    slightly_moved.rssi = -58;
    assert!(tracker.observe(&slightly_moved).is_none());

    // Significant RSSI change (>= 6 dB): emitted
    let mut moved = obs;
    moved.rssi = -52;
    assert!(tracker.observe(&moved).is_some());

    // After 5 ticks without RSSI change: emitted
    for _ in 0..5 {
        tracker.tick();
    }
    assert!(tracker.observe(&moved).is_some());
}

#[test]
fn tracker_is_bounded_and_evicts_oldest() {
    let mut tracker = AssocTracker::new();
    let bssid = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];

    for i in 0..ASSOC_CAPACITY {
        let obs = AssocObservation {
            bssid,
            client: [0x00, 0x10, 0x20, 0x30, 0x40, i as u8],
            channel: 1,
            rssi: -70,
        };
        assert!(tracker.observe(&obs).is_some());
        tracker.tick();
    }
    assert_eq!(tracker.len(), ASSOC_CAPACITY);

    // Adding 33rd entry must evict the oldest entry (i = 0)
    let new_obs = AssocObservation {
        bssid,
        client: [0x00, 0xFF, 0xEE, 0xDD, 0xCC, 0xBB],
        channel: 11,
        rssi: -50,
    };
    assert!(tracker.observe(&new_obs).is_some());
    assert_eq!(tracker.len(), ASSOC_CAPACITY);
}
