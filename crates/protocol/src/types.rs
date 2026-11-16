//! Message definitions. These are plain data: no logic lives here.
//!
//! `#[derive(Serialize, Deserialize)]` makes serde generate the encoding code for us.
//! The ORDER of enum variants is part of the wire format (variant index = tag byte),
//! so only ever append new variants at the end.
use heapless::String;
use serde::{Deserialize, Serialize};

/// Wire format version, the first byte of every packet.
pub const PROTO_VERSION: u8 = 1;
/// Smallest ATT MTU we require so a packet never needs splitting.
pub const MIN_MTU: u16 = 64;
/// Largest postcard body we ever produce.
pub const MAX_BODY: usize = 128;
/// Bytes before the body: `[version][sequence]`.
pub const HEADER_LEN: usize = 2;

/// App -> device.
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
    /// `heapless::Vec<u8, 96>` is a Vec with a fixed capacity of 96 and no heap allocation.
    OtaChunk {
        offset: u32,
        data: heapless::Vec<u8, 96>,
    },
    OtaEnd,
}

/// Device -> app.
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
