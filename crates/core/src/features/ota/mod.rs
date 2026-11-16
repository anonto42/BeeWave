//! OTA feature: strictly sequential, acked, resumable-by-offset update state machine.
use crate::ports::{OtaError, OtaWriter};
use walpad_protocol::{DeviceCommand, TelemetryMessage};

#[derive(Debug, PartialEq, Eq)]
enum State {
    Idle,
    Receiving { size: u32, sha: [u8; 32], next: u32 },
}

pub struct OtaSession<W: OtaWriter> {
    writer: W,
    state: State,
}

impl<W: OtaWriter> OtaSession<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            state: State::Idle,
        }
    }

    /// None if not an OTA command. Some(Ok(Some(ack))) to ack a chunk.
    pub fn handle(
        &mut self,
        cmd: &DeviceCommand,
    ) -> Option<Result<Option<TelemetryMessage>, OtaError>> {
        Some(match cmd {
            DeviceCommand::OtaBegin { size, sha256 } => self.begin(*size, *sha256),
            DeviceCommand::OtaChunk { offset, data } => self.chunk(*offset, data),
            DeviceCommand::OtaEnd => self.end(),
            _ => return None,
        })
    }

    fn begin(&mut self, size: u32, sha: [u8; 32]) -> Result<Option<TelemetryMessage>, OtaError> {
        if self.state != State::Idle {
            self.writer.abort();
        }
        self.writer.begin(size)?;
        self.state = State::Receiving { size, sha, next: 0 };
        Ok(Some(TelemetryMessage::OtaAck { next_offset: 0 }))
    }

    fn chunk(&mut self, offset: u32, data: &[u8]) -> Result<Option<TelemetryMessage>, OtaError> {
        let State::Receiving { size, next, .. } = &mut self.state else {
            return Err(OtaError::BadState);
        };
        if offset != *next {
            return Ok(Some(TelemetryMessage::OtaAck { next_offset: *next }));
        } // client resyncs
        if *next as u64 + data.len() as u64 > *size as u64 {
            return Err(OtaError::TooLarge);
        }
        self.writer.write(data)?;
        *next += data.len() as u32;
        Ok(Some(TelemetryMessage::OtaAck { next_offset: *next }))
    }

    fn end(&mut self) -> Result<Option<TelemetryMessage>, OtaError> {
        let State::Receiving { size, sha, next } = self.state else {
            return Err(OtaError::BadState);
        };
        if next != size {
            return Err(OtaError::BadOffset);
        }
        let r = self.writer.finish(&sha);
        self.state = State::Idle;
        r.map(|_| Some(TelemetryMessage::OtaAck { next_offset: size }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct W {
        written: u32,
        done: bool,
    }
    impl OtaWriter for W {
        fn begin(&mut self, _: u32) -> Result<(), OtaError> {
            self.written = 0;
            Ok(())
        }
        fn write(&mut self, d: &[u8]) -> Result<(), OtaError> {
            self.written += d.len() as u32;
            Ok(())
        }
        fn finish(&mut self, _: &[u8; 32]) -> Result<(), OtaError> {
            self.done = true;
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
    #[test]
    fn happy_path_and_resync() {
        let mut o = OtaSession::new(W::default());
        o.handle(&DeviceCommand::OtaBegin {
            size: 10,
            sha256: [0; 32],
        })
        .unwrap()
        .unwrap();
        // wrong offset -> ack tells client where we are
        assert_eq!(
            o.handle(&chunk(5, 5)),
            Some(Ok(Some(TelemetryMessage::OtaAck { next_offset: 0 })))
        );
        assert_eq!(
            o.handle(&chunk(0, 6)),
            Some(Ok(Some(TelemetryMessage::OtaAck { next_offset: 6 })))
        );
        assert_eq!(o.handle(&chunk(6, 5)), Some(Err(OtaError::TooLarge)));
        assert_eq!(
            o.handle(&DeviceCommand::OtaEnd),
            Some(Err(OtaError::BadOffset))
        );
        assert_eq!(
            o.handle(&chunk(6, 4)),
            Some(Ok(Some(TelemetryMessage::OtaAck { next_offset: 10 })))
        );
        assert!(matches!(o.handle(&DeviceCommand::OtaEnd), Some(Ok(_))));
    }
    #[test]
    fn chunk_without_begin_rejected() {
        let mut o = OtaSession::new(W::default());
        assert_eq!(o.handle(&chunk(0, 1)), Some(Err(OtaError::BadState)));
    }
}
