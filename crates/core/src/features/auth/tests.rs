use beewave_protocol::TelemetryMessage;

use super::{proof, AuthGate};

const KEY: [u8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31,
];
const NONCE: [u8; 16] = [
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
];

fn ok(m: TelemetryMessage) -> bool {
    matches!(m, TelemetryMessage::AuthResult { ok: true })
}

/// Known answer computed independently with Python's `hmac`; the web test uses the same value.
#[test]
fn proof_matches_known_answer() {
    let mac = proof(&KEY, &NONCE).unwrap();
    let hex: std::string::String = mac.iter().map(|b| std::format!("{b:02x}")).collect();
    assert_eq!(
        hex,
        "7395f8286d558150f51cdba47200dcd4020f5a5abf2748d8170071d24158ca02"
    );
}

#[test]
fn correct_proof_authenticates() {
    let mut g = AuthGate::new(Some(KEY));
    assert!(!g.is_authenticated());
    g.begin(NONCE);
    assert!(ok(g.prove(&proof(&KEY, &NONCE).unwrap())));
    assert!(g.is_authenticated());
}

#[test]
fn wrong_proof_is_rejected() {
    let mut g = AuthGate::new(Some(KEY));
    g.begin(NONCE);
    assert!(!ok(g.prove(&[0; 32])));
    assert!(!g.is_authenticated());
}

#[test]
fn proof_for_another_key_is_rejected() {
    let mut g = AuthGate::new(Some(KEY));
    g.begin(NONCE);
    assert!(!ok(g.prove(&proof(&[9; 32], &NONCE).unwrap())));
}

#[test]
fn proof_without_begin_is_rejected() {
    let mut g = AuthGate::new(Some(KEY));
    assert!(!ok(g.prove(&proof(&KEY, &NONCE).unwrap())));
}

#[test]
fn proof_cannot_be_replayed() {
    let mut g = AuthGate::new(Some(KEY));
    let good = proof(&KEY, &NONCE).unwrap();
    g.begin(NONCE);
    assert!(ok(g.prove(&good)));
    g.reset();
    g.begin([1; 16]); // a new challenge makes the old proof useless
    assert!(!ok(g.prove(&good)));
    assert!(!g.is_authenticated());
}

#[test]
fn nonce_is_single_use() {
    let mut g = AuthGate::new(Some(KEY));
    g.begin(NONCE);
    assert!(!ok(g.prove(&[0; 32]))); // failed attempt burns the nonce
    assert!(!ok(g.prove(&proof(&KEY, &NONCE).unwrap())));
}

#[test]
fn locked_gate_never_authenticates() {
    let mut g = AuthGate::new(None);
    g.begin(NONCE);
    assert!(!ok(g.prove(&proof(&KEY, &NONCE).unwrap())));
    assert!(!g.is_authenticated());
}

#[test]
fn reset_ends_the_session() {
    let mut g = AuthGate::new(Some(KEY));
    g.begin(NONCE);
    g.prove(&proof(&KEY, &NONCE).unwrap());
    g.reset();
    assert!(!g.is_authenticated());
}

#[test]
fn an_unclaimed_gate_can_be_claimed_once() {
    let mut g = AuthGate::new(None);
    assert!(!g.is_claimed());
    assert!(g.claim(KEY));
    assert!(g.is_claimed() && g.is_authenticated());
    assert!(!g.claim([9; 32]), "second claim is refused");
    // the original key still verifies
    g.reset();
    g.begin(NONCE);
    assert!(ok(g.prove(&proof(&KEY, &NONCE).unwrap())));
}

#[test]
fn the_all_zero_key_cannot_claim() {
    let mut g = AuthGate::new(None);
    assert!(!g.claim([0; 32]));
    assert!(!g.is_claimed());
}
