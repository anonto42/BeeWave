//! Encoding and decoding of packets.
//!
//! `encode`/`decode` are generic: `T: Serialize` means "any type serde can encode".
//! The four public functions below just pin `T` to our two message types.
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::types::{DeviceCommand, TelemetryMessage, HEADER_LEN, PROTO_VERSION};

fn encode<T: Serialize>(seq: u8, msg: &T, out: &mut [u8]) -> Result<usize, Error> {
    if out.len() < HEADER_LEN {
        return Err(Error::TooShort);
    }
    out[0] = PROTO_VERSION;
    out[1] = seq;
    // `?` would also work; map_err converts postcard's error into ours.
    let body = postcard::to_slice(msg, &mut out[HEADER_LEN..]).map_err(|_| Error::Codec)?;
    Ok(HEADER_LEN + body.len())
}

// The `'a` lifetime says: the decoded value may borrow from `buf`.
fn decode<'a, T: Deserialize<'a>>(buf: &'a [u8]) -> Result<(u8, T), Error> {
    if buf.len() < HEADER_LEN {
        return Err(Error::TooShort);
    }
    if buf[0] != PROTO_VERSION {
        return Err(Error::BadVersion(buf[0]));
    }
    let msg = postcard::from_bytes(&buf[HEADER_LEN..]).map_err(|_| Error::Codec)?;
    Ok((buf[1], msg))
}

pub fn encode_command(seq: u8, c: &DeviceCommand, out: &mut [u8]) -> Result<usize, Error> {
    encode(seq, c, out)
}
pub fn decode_command(buf: &[u8]) -> Result<(u8, DeviceCommand), Error> {
    decode(buf)
}
pub fn encode_telemetry(seq: u8, t: &TelemetryMessage, out: &mut [u8]) -> Result<usize, Error> {
    encode(seq, t, out)
}
pub fn decode_telemetry(buf: &[u8]) -> Result<(u8, TelemetryMessage), Error> {
    decode(buf)
}
