extern crate std;
use std::{cell::RefCell, rc::Rc, vec};

use beewave_protocol::{DeviceCommand, TelemetryMessage};
use ed25519_dalek::{Signer, SigningKey};

use super::*;
use crate::ports::{OtaError, OtaWriter};

#[derive(Default)]
struct Log {
    finished: u32,
    aborted: u32,
}

#[derive(Default)]
struct FakeWriter(Rc<RefCell<Log>>);

impl OtaWriter for FakeWriter {
    fn begin(&mut self, _: u32) -> Result<(), OtaError> {
        Ok(())
    }
    fn write(&mut self, _: &[u8]) -> Result<(), OtaError> {
        Ok(())
    }
    fn finish(&mut self, _: &[u8; 32]) -> Result<(), OtaError> {
        self.0.borrow_mut().finished += 1;
        Ok(())
    }
    fn abort(&mut self) {
        self.0.borrow_mut().aborted += 1;
    }
}

const DIGEST: [u8; 32] = [0x5a; 32];

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[11; 32])
}

fn session() -> (OtaSession<FakeWriter>, Rc<RefCell<Log>>) {
    let log = Rc::new(RefCell::new(Log::default()));
    let key = signing_key().verifying_key().to_bytes();
    (OtaSession::new(FakeWriter(log.clone()), Some(key)), log)
}

fn begin(o: &mut OtaSession<FakeWriter>, size: u32) {
    o.handle(&DeviceCommand::OtaBegin {
        size,
        sha256: DIGEST,
    })
    .unwrap()
    .unwrap();
}

fn chunk(offset: u32, n: usize) -> DeviceCommand {
    DeviceCommand::OtaChunk {
        offset,
        data: heapless::Vec::from_slice(&vec![0u8; n]).unwrap(),
    }
}

fn ack(next_offset: u32) -> Option<Result<Option<TelemetryMessage>, OtaError>> {
    Some(Ok(Some(TelemetryMessage::OtaAck { next_offset })))
}

fn end_signed(digest: &[u8; 32], key: &SigningKey) -> DeviceCommand {
    let sig = key.sign(digest).to_bytes();
    DeviceCommand::OtaEndSigned {
        sig_r: sig[..32].try_into().unwrap(),
        sig_s: sig[32..].try_into().unwrap(),
    }
}

#[test]
fn signed_update_completes() {
    let (mut o, log) = session();
    begin(&mut o, 10);
    assert_eq!(o.handle(&chunk(0, 10)), ack(10));
    assert_eq!(o.handle(&end_signed(&DIGEST, &signing_key())), ack(10));
    assert_eq!(log.borrow().finished, 1);
}

#[test]
fn chunks_resync_and_are_bounded() {
    let (mut o, _) = session();
    begin(&mut o, 10);
    assert_eq!(o.handle(&chunk(5, 5)), ack(0)); // wrong offset: ack says where we are
    assert_eq!(o.handle(&chunk(0, 6)), ack(6));
    assert_eq!(o.handle(&chunk(6, 5)), Some(Err(OtaError::TooLarge)));
    assert_eq!(
        o.handle(&end_signed(&DIGEST, &signing_key())),
        Some(Err(OtaError::BadOffset))
    );
    assert_eq!(o.handle(&chunk(6, 4)), ack(10));
}

#[test]
fn chunk_without_begin_rejected() {
    let (mut o, _) = session();
    assert_eq!(o.handle(&chunk(0, 1)), Some(Err(OtaError::BadState)));
}

#[test]
fn unsigned_end_is_rejected_and_never_finishes() {
    let (mut o, log) = session();
    begin(&mut o, 4);
    o.handle(&chunk(0, 4));
    assert_eq!(
        o.handle(&DeviceCommand::OtaEnd),
        Some(Err(OtaError::Unsigned))
    );
    assert_eq!(log.borrow().finished, 0);
    assert!(log.borrow().aborted >= 1);
}

#[test]
fn bad_signature_is_rejected_and_aborts() {
    let (mut o, log) = session();
    begin(&mut o, 4);
    o.handle(&chunk(0, 4));
    let forged = SigningKey::from_bytes(&[99; 32]); // someone else's key
    assert_eq!(
        o.handle(&end_signed(&DIGEST, &forged)),
        Some(Err(OtaError::SignatureInvalid))
    );
    assert_eq!(log.borrow().finished, 0);
    assert!(log.borrow().aborted >= 1);
    // The session is gone, so the rejected image cannot be finished afterwards.
    assert_eq!(o.handle(&chunk(4, 1)), Some(Err(OtaError::BadState)));
}

#[test]
fn signature_over_a_different_image_is_rejected() {
    let (mut o, log) = session();
    begin(&mut o, 4);
    o.handle(&chunk(0, 4));
    assert_eq!(
        o.handle(&end_signed(&[0x11; 32], &signing_key())),
        Some(Err(OtaError::SignatureInvalid))
    );
    assert_eq!(log.borrow().finished, 0);
}

#[test]
fn device_without_a_key_refuses_updates() {
    let log = Rc::new(RefCell::new(Log::default()));
    let mut o = OtaSession::new(FakeWriter(log), None);
    assert_eq!(
        o.handle(&DeviceCommand::OtaBegin {
            size: 4,
            sha256: DIGEST
        }),
        Some(Err(OtaError::Unsigned))
    );
}

fn hex(s: &str) -> std::vec::Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

/// A signature made by `openssl pkeyutl -sign -rawin` (what `make fw-image` uses) verifies.
#[test]
fn accepts_a_signature_made_by_openssl() {
    let public: [u8; 32] = hex("a15d5a7ced079882f0db3ba491a447c7c10a6d0fc706d682c62f9c738e33710c")
        .try_into()
        .unwrap();
    let digest: [u8; 32] = hex("1da9bc8bb6069ad0a4170164058a8d3210cdf5aa22694e54e7f626b4c3db5878")
        .try_into()
        .unwrap();
    let sig = hex(
        "906114ddc985a8d72ec61d4daaba243fa21d563564a78adef12aed6b89013547\
         e0bdab23478753ec792b8c0f63b86bb3e1bd15a7378dd3566efc7fe1869aed05",
    );
    assert!(signature::verify(
        &public,
        &digest,
        &sig[..32].try_into().unwrap(),
        &sig[32..].try_into().unwrap()
    ));
    assert!(!signature::verify(
        &public,
        &[0; 32],
        &sig[..32].try_into().unwrap(),
        &sig[32..].try_into().unwrap()
    ));
}
