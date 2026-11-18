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
pub use types::{
    AdClass, AlertSettings, AnomalyKind, DeviceCommand, HopOrder, LogLevel, RadioSettings,
    RejectReason, ScanSettings, TelemetryMessage, TelemetrySettings, WifiPmfMode,
    WifiSecurityProtocol, AUTH_CONTEXT, AUTH_NONCE_LEN, HEADER_LEN, HOP_LIST_LEN, MAX_BODY,
    MAX_EAPOL_CAPTURE, MAX_SSID_FILTER, MIN_MTU, PROTO_VERSION, SETTINGS_SCHEMA,
};
