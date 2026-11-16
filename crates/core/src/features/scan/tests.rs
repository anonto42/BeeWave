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
