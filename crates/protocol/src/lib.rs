//! Wire protocol shared by firmware and PWA.
//!
//! Every BLE write/notification carries exactly one packet:
//!   [ver: u8][seq: u8][postcard body]
//! Messages are required to fit in one ATT payload (MTU >= 64 negotiated),
//! so there is no fragmentation layer.
#![cfg_attr(not(feature = "std"), no_std)]

use heapless::String;
use serde::{Deserialize, Serialize};

pub const PROTO_VERSION: u8 = 1;
pub const MIN_MTU: u16 = 64;
pub const MAX_BODY: usize = 128;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DeviceCommand {
    StartScan {
        band_mask: u8,
    },
    StopOperation,
    ConfigureRadio {
        channel: u8,
    },
    RebootDevice,
    OtaBegin {
        size: u32,
        sha256: [u8; 32],
    },
    OtaChunk {
        offset: u32,
        data: heapless::Vec<u8, 96>,
    },
    OtaEnd,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TelemetryMessage {
    Status {
        active_channel: u8,
        free_heap: u32,
        cpu_load: u8,
        dropped: u16,
    },
    ScanResult {
        bssid: [u8; 6],
        channel: u8,
        rssi: i8,
        ssid: String<32>,
    },
    Event {
        code: u16,
        message: String<64>,
    },
    OtaAck {
        next_offset: u32,
    },
}

#[derive(Debug, PartialEq)]
pub enum Error {
    BadVersion(u8),
    TooShort,
    Codec,
}

fn encode<T: Serialize>(seq: u8, msg: &T, out: &mut [u8]) -> Result<usize, Error> {
    if out.len() < 2 {
        return Err(Error::TooShort);
    }
    out[0] = PROTO_VERSION;
    out[1] = seq;
    let body = postcard::to_slice(msg, &mut out[2..]).map_err(|_| Error::Codec)?;
    Ok(2 + body.len())
}

fn decode<'a, T: Deserialize<'a>>(buf: &'a [u8]) -> Result<(u8, T), Error> {
    if buf.len() < 2 {
        return Err(Error::TooShort);
    }
    if buf[0] != PROTO_VERSION {
        return Err(Error::BadVersion(buf[0]));
    }
    Ok((
        buf[1],
        postcard::from_bytes(&buf[2..]).map_err(|_| Error::Codec)?,
    ))
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
