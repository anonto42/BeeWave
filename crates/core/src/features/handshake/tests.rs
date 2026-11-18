use heapless::Vec;

use super::{parse_eapol, HandshakeTracker};

const BSSID: [u8; 6] = [0xAA; 6];
const CLIENT: [u8; 6] = [0xBB; 6];

/// Builds a minimal 802.11 data frame carrying one EAPOL-Key message, with the given key-info bits and
/// an all-`fill` key_data region of `key_data_len` bytes appended after the fixed EAPOL-Key body.
fn frame(
    key_info: u16,
    from_ds: bool,
    replay_counter: u64,
    key_data_len: usize,
    fill: u8,
) -> Vec<u8, 400> {
    let mut f: Vec<u8, 400> = Vec::new();
    let fc1 = if from_ds { 0x02 } else { 0x01 }; // FromDS or ToDS
    f.extend_from_slice(&[0x08, fc1]).unwrap(); // frame control: plain Data
    f.extend_from_slice(&[0; 2]).unwrap(); // duration
    let (addr1, addr2) = if from_ds {
        (CLIENT, BSSID)
    } else {
        (BSSID, CLIENT)
    };
    f.extend_from_slice(&addr1).unwrap();
    f.extend_from_slice(&addr2).unwrap();
    f.extend_from_slice(&BSSID).unwrap(); // addr3
    f.extend_from_slice(&[0; 2]).unwrap(); // sequence control
    f.extend_from_slice(&[0xAA, 0xAA, 0x03, 0x00, 0x00, 0x00, 0x88, 0x8E])
        .unwrap(); // LLC/SNAP + EtherType
    f.extend_from_slice(&[1, 3, 0, 0]).unwrap(); // EAPOL version, type=Key, length (unchecked)
    f.push(2).unwrap(); // descriptor type: RSN
    f.extend_from_slice(&key_info.to_be_bytes()).unwrap();
    f.extend_from_slice(&[0; 2]).unwrap(); // key length
    f.extend_from_slice(&replay_counter.to_be_bytes()).unwrap();
    f.extend_from_slice(&[0x11; 32]).unwrap(); // nonce
    f.extend_from_slice(&[0; 16]).unwrap(); // IV
    f.extend_from_slice(&[0; 8]).unwrap(); // RSC
    f.extend_from_slice(&[0; 8]).unwrap(); // reserved
    f.extend_from_slice(&[0x22; 16]).unwrap(); // MIC
    f.extend_from_slice(&(key_data_len as u16).to_be_bytes())
        .unwrap();
    for _ in 0..key_data_len {
        f.push(fill).unwrap();
    }
    f
}

const PAIRWISE: u16 = 1 << 3;
const MSG1: u16 = PAIRWISE | (1 << 7); // ACK
const MSG2: u16 = PAIRWISE | (1 << 8); // MIC
const MSG3: u16 = PAIRWISE | (1 << 7) | (1 << 8) | (1 << 9); // ACK + MIC + Secure
const MSG4: u16 = PAIRWISE | (1 << 8) | (1 << 9); // MIC + Secure

#[test]
fn classifies_each_handshake_message() {
    for (info, want) in [(MSG1, 1), (MSG2, 2), (MSG3, 3), (MSG4, 4)] {
        let f =
            parse_eapol(&frame(info, true, 1, 0, 0)).unwrap_or_else(|| panic!("info={info:#x}"));
        assert_eq!(f.message, want, "info={info:#x}");
        assert_eq!((f.bssid, f.client), (BSSID, CLIENT));
    }
}

#[test]
fn rejects_key_info_that_matches_no_message() {
    assert!(parse_eapol(&frame(0, true, 1, 0, 0)).is_none());
    // a non-pairwise (group key) frame with the same ACK/MIC/Secure bits as message 3 is not message 3
    let group_like_msg3 = (1 << 7) | (1 << 8) | (1 << 9); // no PAIRWISE bit
    assert!(parse_eapol(&frame(group_like_msg3, true, 1, 0, 0)).is_none());
    let group_like_msg4 = (1 << 8) | (1 << 9); // no PAIRWISE bit
    assert!(parse_eapol(&frame(group_like_msg4, true, 1, 0, 0)).is_none());
}

#[test]
fn reads_the_replay_counter() {
    let f = parse_eapol(&frame(MSG1, true, 0xDEAD_BEEF, 0, 0)).unwrap();
    assert_eq!(f.replay_counter, 0xDEAD_BEEF);
}

#[test]
fn to_ds_and_from_ds_give_the_same_bssid_and_client() {
    let a = parse_eapol(&frame(MSG2, false, 1, 0, 0)).unwrap(); // client -> AP
    let b = parse_eapol(&frame(MSG1, true, 1, 0, 0)).unwrap(); // AP -> client
    assert_eq!((a.bssid, a.client), (b.bssid, b.client));
}

#[test]
fn a_long_frame_is_capped_and_flagged_truncated() {
    let short = parse_eapol(&frame(MSG1, true, 1, 0, 0)).unwrap();
    assert!(!short.truncated);
    let long = parse_eapol(&frame(MSG3, true, 1, 200, 0x55)).unwrap();
    assert!(long.truncated);
    assert_eq!(long.eapol.len(), beewave_protocol::MAX_EAPOL_CAPTURE);
}

#[test]
fn rejects_non_data_and_non_eapol_frames() {
    let mut not_data = frame(MSG1, true, 1, 0, 0);
    not_data[0] = 0x80; // beacon, not a data frame
    assert!(parse_eapol(&not_data).is_none());

    let mut wrong_ethertype = frame(MSG1, true, 1, 0, 0);
    wrong_ethertype[30] = 0x00; // corrupt the EtherType bytes (LLC/SNAP starts at offset 24)
    assert!(parse_eapol(&wrong_ethertype).is_none());

    let mut wds = frame(MSG1, true, 1, 0, 0);
    wds[1] |= 0x01; // set ToDS as well as FromDS: four-address WDS frame
    assert!(parse_eapol(&wds).is_none());

    assert!(parse_eapol(&[0u8; 10]).is_none());
}

#[test]
fn tracker_reports_completion_once_per_pair() {
    let mut t = HandshakeTracker::new();
    let m1 = parse_eapol(&frame(MSG1, true, 1, 0, 0)).unwrap();
    let m2 = parse_eapol(&frame(MSG2, false, 2, 0, 0)).unwrap();
    assert!(t.observe(&m1).is_none(), "message 1 alone is not enough");
    let done = t.observe(&m2).expect("message 2 completes the pair");
    assert_eq!((done.bssid, done.client), (BSSID, CLIENT));
    assert!(t.observe(&m2).is_none(), "reported only once");
}

#[test]
fn message_2_plus_3_also_completes_a_pair() {
    let mut t = HandshakeTracker::new();
    t.observe(&parse_eapol(&frame(MSG2, false, 1, 0, 0)).unwrap());
    let done = t.observe(&parse_eapol(&frame(MSG3, true, 2, 0, 0)).unwrap());
    assert!(done.is_some());
}

#[test]
fn different_pairs_are_tracked_independently() {
    let mut t = HandshakeTracker::new();
    let mut other = frame(MSG1, true, 1, 0, 0);
    other[10] = 0xCC; // a different BSSID (addr2, since FromDS)
    let a = parse_eapol(&frame(MSG1, true, 1, 0, 0)).unwrap();
    let b = parse_eapol(&other).unwrap();
    assert_ne!(a.bssid, b.bssid);
    assert!(t.observe(&a).is_none());
    assert!(t.observe(&b).is_none(), "a different pair starts fresh");
}

#[test]
fn reset_forgets_every_pair() {
    let mut t = HandshakeTracker::new();
    let m1 = parse_eapol(&frame(MSG1, true, 1, 0, 0)).unwrap();
    let m2 = parse_eapol(&frame(MSG2, false, 2, 0, 0)).unwrap();
    t.observe(&m1);
    t.observe(&m2);
    t.reset();
    assert!(t.observe(&m1).is_none());
    assert!(
        t.observe(&m2).is_some(),
        "completes again after reset, as if new"
    );
}

#[test]
fn the_table_is_bounded_and_keeps_working_once_full() {
    let mut t = HandshakeTracker::new();
    for i in 0..(super::CAPACITY as u8 + 4) {
        let mut f = frame(MSG1, true, 1, 0, 0);
        f[4] = i; // vary the client address (addr1, FromDS)
        t.observe(&parse_eapol(&f).unwrap());
    }
    // more pairs than CAPACITY were inserted with no panic; a brand new pair still completes normally,
    // which it could not if eviction had left the table stuck full or corrupted
    let mut m1 = frame(MSG1, true, 1, 0, 0);
    m1[4] = 200;
    let mut m2 = frame(MSG2, false, 2, 0, 0);
    m2[10] = 200;
    assert!(t.observe(&parse_eapol(&m1).unwrap()).is_none());
    assert!(t.observe(&parse_eapol(&m2).unwrap()).is_some());
}
