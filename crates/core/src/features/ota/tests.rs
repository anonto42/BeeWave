use beewave_protocol::{DeviceCommand, TelemetryMessage};

use super::*;
use crate::ports::{OtaError, OtaWriter};

#[derive(Default)]
struct FakeWriter {
    written: u32,
}

impl OtaWriter for FakeWriter {
    fn begin(&mut self, _: u32) -> Result<(), OtaError> {
        self.written = 0;
        Ok(())
    }
    fn write(&mut self, d: &[u8]) -> Result<(), OtaError> {
        self.written += d.len() as u32;
        Ok(())
    }
    fn finish(&mut self, _: &[u8; 32]) -> Result<(), OtaError> {
        Ok(())
    }
    fn abort(&mut self) {}
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

#[test]
fn happy_path_and_resync() {
    let mut o = OtaSession::new(FakeWriter::default());
    o.handle(&DeviceCommand::OtaBegin {
        size: 10,
        sha256: [0; 32],
    })
    .unwrap()
    .unwrap();
    assert_eq!(o.handle(&chunk(5, 5)), ack(0)); // wrong offset: ack says where we are
    assert_eq!(o.handle(&chunk(0, 6)), ack(6));
    assert_eq!(o.handle(&chunk(6, 5)), Some(Err(OtaError::TooLarge)));
    assert_eq!(
        o.handle(&DeviceCommand::OtaEnd),
        Some(Err(OtaError::BadOffset))
    );
    assert_eq!(o.handle(&chunk(6, 4)), ack(10));
    assert!(matches!(o.handle(&DeviceCommand::OtaEnd), Some(Ok(_))));
}

#[test]
fn chunk_without_begin_rejected() {
    let mut o = OtaSession::new(FakeWriter::default());
    assert_eq!(o.handle(&chunk(0, 1)), Some(Err(OtaError::BadState)));
}
