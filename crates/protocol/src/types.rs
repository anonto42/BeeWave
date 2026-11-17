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

/// Domain separator prepended to the nonce before signing (the ASCII bytes of this string).
pub const AUTH_CONTEXT: &[u8] = b"beewave-auth-v1";
/// Length of the authentication nonce.
pub const AUTH_NONCE_LEN: usize = 16;

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
    /// Ask the device to report its saved settings.
    GetSettings,
    /// Save settings to flash and apply them. `channel` 0 = auto-hop.
    SetSettings {
        channel: u8,
        auto_scan: bool,
    },
    /// Start authentication; the device answers with `AuthChallenge`.
    AuthBegin,
    /// Prove knowledge of the shared key: `HMAC-SHA256(key, AUTH_CONTEXT || nonce)`.
    AuthProve {
        mac: [u8; 32],
    },
    /// Finish an update, proving the image is signed. The Ed25519 signature (64 bytes) is over
    /// the image SHA-256 given in `OtaBegin`; serde cannot encode `[u8; 64]`, so it is sent as
    /// its two 32-byte halves (`R` then `S`).
    OtaEndSigned {
        sig_r: [u8; 32],
        sig_s: [u8; 32],
    },
    /// Claim an unclaimed device: the key it will require from now on. Refused when the device
    /// already has a key.
    Provision {
        key: [u8; 32],
    },
    /// Ask for the current monitor configuration (answered with `MonitorConfig`).
    GetMonitor,
    /// Configure the receive-only monitors (answered with `MonitorConfig`). Never changes what the
    /// radio transmits: nothing in BeeWave transmits on Wi-Fi, and the BLE scan is passive.
    SetMonitor {
        /// Deauth frames per second per BSSID that raise a `DeauthAlert`; 0 turns the detector off.
        deauth_threshold: u8,
        /// Run the passive BLE advertisement monitor.
        ble_enabled: bool,
        /// A class spikes when its count exceeds its baseline times this (2..=20).
        ble_spike_factor: u8,
    },
}

/// Class of a received BLE advertisement (receive-only monitor). Append new classes at the end.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdClass {
    /// Google Fast Pair (service data 0xFE2C).
    FastPair,
    /// Apple iBeacon.
    IBeacon,
    /// Apple proximity pairing (AirPods-style).
    Proximity,
    /// Anything else.
    Generic,
}

/// What an `AnomalyAlert` is about. Append new kinds at the end.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnomalyKind {
    /// A known SSID is now advertised by a BSSID not seen with it before.
    NewBssidForSsid,
    /// The same BSSID was received on two channels within a minute.
    TwoChannels,
    /// RSSI jumped by more than 20 dB between cycles (`detail` = the jump in dB, capped at 255).
    RssiOutlier,
    /// A network came back after missing more than 5 cycles (`detail` = cycles missed, capped at 255).
    Returned,
}

/// Device -> app.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TelemetryMessage {
    Status {
        active_channel: u8,
        free_heap: u32,
        cpu_load: u8,
        dropped: u16,
        /// True while the sniffer is running (scan in progress).
        scanning: bool,
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
    /// Current saved settings (reply to GetSettings / SetSettings).
    Settings {
        channel: u8,
        auto_scan: bool,
    },
    /// Fresh single-use nonce the app must sign (reply to `AuthBegin`).
    AuthChallenge {
        nonce: [u8; 16],
    },
    /// Outcome of `AuthProve`.
    AuthResult {
        ok: bool,
    },
    /// Reply to `AuthBegin` while the device has no key yet: the client may `Provision` one.
    Unclaimed,
    /// Deauthentication frames from one BSSID reached the threshold within a second.
    DeauthAlert {
        bssid: [u8; 6],
        count: u16,
    },
    /// BLE advertisements per class over the last 60 s (sent every 10 s while the monitor runs).
    BleStats {
        fast_pair: u16,
        ibeacon: u16,
        proximity: u16,
        generic: u16,
    },
    /// One advertisement class is far above its baseline.
    BleSpike {
        class: AdClass,
        count: u16,
        baseline: u16,
    },
    /// Something odd about a network (see `AnomalyKind`).
    AnomalyAlert {
        kind: AnomalyKind,
        bssid: [u8; 6],
        detail: u8,
    },
    /// The current monitor configuration (reply to `GetMonitor` / `SetMonitor`).
    MonitorConfig {
        deauth_threshold: u8,
        ble_enabled: bool,
        ble_spike_factor: u8,
    },
}
