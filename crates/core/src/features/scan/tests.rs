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
    assert!(t.observe(obs(-50)).is_some());
    assert!(t.observe(obs(-52)).is_none());
    assert!(t.observe(obs(-60)).is_some());
}

#[test]
fn survives_overflow() {
    let mut t = ScanTable::new();
    for i in 0..200u8 {
        let mut o = obs(-50);
        o.bssid = [i, 0, 0, 0, 0, 1];
        assert!(t.observe(o).is_some());
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
