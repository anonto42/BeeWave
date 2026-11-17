use beewave_protocol::{DeviceCommand, TelemetryMessage};

use super::signature;
use super::state::State;
use crate::ports::{OtaError, OtaWriter};

/// Result of handling one OTA command: optionally an ack to send back.
type Outcome = Result<Option<TelemetryMessage>, OtaError>;

pub struct OtaSession<W: OtaWriter> {
    writer: W,
    state: State,
    /// Ed25519 public key updates must be signed with. `None` = no key, updates are refused.
    signing_key: Option<[u8; 32]>,
}

impl<W: OtaWriter> OtaSession<W> {
    pub fn new(writer: W, signing_key: Option<[u8; 32]>) -> Self {
        Self {
            writer,
            state: State::Idle,
            signing_key,
        }
    }

    /// `None` if the command is not an OTA command.
    pub fn handle(&mut self, cmd: &DeviceCommand) -> Option<Outcome> {
        Some(match cmd {
            DeviceCommand::OtaBegin { size, sha256 } => self.begin(*size, *sha256),
            DeviceCommand::OtaChunk { offset, data } => self.chunk(*offset, data),
            DeviceCommand::OtaEnd => self.end_unsigned(),
            DeviceCommand::OtaEndSigned { sig_r, sig_s } => self.end_signed(sig_r, sig_s),
            _ => return None,
        })
    }

    /// Drop any update in flight (the client went away).
    /// Is an update being received right now?
    pub fn in_progress(&self) -> bool {
        self.state != super::state::State::Idle
    }

    pub fn abort(&mut self) {
        if self.state != State::Idle {
            self.writer.abort();
            self.state = State::Idle;
        }
    }

    fn begin(&mut self, size: u32, sha: [u8; 32]) -> Outcome {
        if self.signing_key.is_none() {
            return Err(OtaError::Unsigned); // fail closed: nothing could ever verify
        }
        if self.state != State::Idle {
            self.writer.abort(); // a new begin restarts any update in flight
        }
        self.writer.begin(size)?;
        self.state = State::Receiving { size, sha, next: 0 };
        Ok(Some(TelemetryMessage::OtaAck { next_offset: 0 }))
    }

    fn chunk(&mut self, offset: u32, data: &[u8]) -> Outcome {
        let State::Receiving { size, next, .. } = &mut self.state else {
            return Err(OtaError::BadState);
        };
        if offset != *next {
            // Wrong offset: don't write, tell the client where we are so it can resync.
            return Ok(Some(TelemetryMessage::OtaAck { next_offset: *next }));
        }
        if *next as u64 + data.len() as u64 > *size as u64 {
            return Err(OtaError::TooLarge);
        }
        self.writer.write(data)?;
        *next += data.len() as u32;
        Ok(Some(TelemetryMessage::OtaAck { next_offset: *next }))
    }

    /// A plain end is never accepted: every update must carry a signature.
    fn end_unsigned(&mut self) -> Outcome {
        self.abort();
        Err(OtaError::Unsigned)
    }

    fn end_signed(&mut self, sig_r: &[u8; 32], sig_s: &[u8; 32]) -> Outcome {
        let State::Receiving { size, sha, next } = self.state else {
            return Err(OtaError::BadState);
        };
        if next != size {
            return Err(OtaError::BadOffset);
        }
        let genuine = self
            .signing_key
            .is_some_and(|key| signature::verify(&key, &sha, sig_r, sig_s));
        if !genuine {
            self.abort(); // never let an unverified image become bootable
            return Err(OtaError::SignatureInvalid);
        }
        let result = self.writer.finish(&sha);
        self.state = State::Idle;
        result.map(|_| Some(TelemetryMessage::OtaAck { next_offset: size }))
    }
}
