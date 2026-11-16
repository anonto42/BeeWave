//! Wire protocol shared by the firmware and the PWA.
//!
//! Every BLE write/notification carries exactly one packet:
//! `[ver: u8][seq: u8][postcard body]`.
//!
//! File map (read in this order):
//! - `types.rs`: the messages and constants (the "what")
//! - `error.rs`: what can go wrong when decoding
//! - `codec.rs`: turning messages into bytes and back (the "how")
#![cfg_attr(not(feature = "std"), no_std)]

mod codec;
mod error;
mod types;

pub use codec::{decode_command, decode_telemetry, encode_command, encode_telemetry};
pub use error::Error;
pub use types::{DeviceCommand, TelemetryMessage, HEADER_LEN, MAX_BODY, MIN_MTU, PROTO_VERSION};
