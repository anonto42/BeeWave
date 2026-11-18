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
    /// Ask for every persisted setting (answered with `SettingsAll`).
    GetSettingsAll,
    /// Validate and persist every setting at once (answered with `SettingsSaved`; nothing is written
    /// when any field is rejected).
    SaveSettingsAll {
        scan: ScanSettings,
        radio: RadioSettings,
        telemetry: TelemetrySettings,
        alerts: AlertSettings,
    },
    /// Erase the stored settings, the claimed key and the revision counter, then reboot. Refused while
    /// an update is in flight.
    FactoryReset,
    /// Run the short diagnostic sequence (answered with `SelfTestResult`).
    RunSelfTest,
    /// NVS usage, partition layout and the settings revision (answered with `StorageInfo`).
    GetStorageInfo,
    /// Firmware version, build, key fingerprint and build-time flags (answered with `DeviceInfo`).
    GetDeviceInfo,
    /// Turn the passive EAPOL handshake listener on or off (answered with `HandshakeCaptureState`). Off by
    /// default; never transmits anything, even while on.
    SetHandshakeCapture {
        enabled: bool,
    },
    /// Ask for the current handshake-capture state.
    GetHandshakeCapture,
}

/// Longest raw EAPOL frame (from its own first byte) kept in one `HandshakeFrame`; longer frames are truncated
/// and flagged, to stay inside `MAX_BODY`. The fixed EAPOL-Key layout alone (header + nonce + MIC, no key
/// data) is 99 bytes, so this must be at least that; it leaves a few bytes of key data on top.
pub const MAX_EAPOL_CAPTURE: usize = 100;

/// Version of the settings layout, sent in `SettingsAll` so an older or newer client can tell.
pub const SETTINGS_SCHEMA: u8 = 1;
/// Longest SSID filter.
pub const MAX_SSID_FILTER: usize = 32;
/// Slots in a custom hop list (0 = unused slot).
pub const HOP_LIST_LEN: usize = 13;

/// Order in which the sniffer visits channels while hopping. Append new variants at the end.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HopOrder {
    /// 1, 2, 3 ... 13.
    Ascending,
    /// The non-overlapping channels first: 1, 6, 11, 2, 7, 12, 3, 8, 13, 4, 9, 5, 10.
    Ascending1_6_11,
    /// The channels in `custom_hop_list`, in that order.
    Custom,
}

/// Verbosity of the device's own log output. Append new variants at the end.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

/// Why a request was refused. Each field-level reason names the field so a client can highlight it.
/// Append new variants at the end.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    ScanIntervalRange,
    DedupRssiRange,
    ChannelMaskRange,
    DefaultChannelRange,
    HopDwellRange,
    /// `hop_order` is Custom but the list has no channel.
    HopListEmpty,
    /// A custom hop list entry is not 0 or 1-13, or a channel appears twice.
    HopListInvalid,
    StatusPeriodRange,
    DeauthThresholdRange,
    DeauthWindowRange,
    /// `deauth_window_ms` is shorter than twice `scan_interval_ms`.
    WindowTooShort,
    /// An update (or a scan) is in progress.
    Busy,
    /// Flash refused the write.
    Storage,
}

/// Scan behaviour, persisted on the device.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ScanSettings {
    pub scan_at_boot: bool,
    /// Minimum time between re-reports of the same network, 100..=10000 ms (new networks are reported at once).
    pub scan_interval_ms: u32,
    /// A known network is re-reported only when RSSI moves by this many dB, 1..=30.
    pub dedup_rssi_db: u8,
    /// Report only networks whose SSID contains this text (case-insensitive); empty = no filter.
    pub ssid_filter: String<MAX_SSID_FILTER>,
    /// Bit 0 = channel 1 ... bit 12 = channel 13. 0 = all channels.
    pub channel_filter_mask: u16,
}

/// Radio behaviour, persisted on the device.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RadioSettings {
    /// 0 = AUTO (hop), else 1..=13 (pinned).
    pub default_channel: u8,
    /// Time on each channel while hopping, 100..=2000 ms.
    pub hop_dwell_ms: u16,
    pub hop_order: HopOrder,
    /// Used when `hop_order` is Custom: channels 1-13 in visiting order, unused slots 0.
    pub custom_hop_list: [u8; HOP_LIST_LEN],
}

/// Telemetry cadence and device log verbosity, persisted on the device.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TelemetrySettings {
    /// Status heartbeat period, 250..=5000 ms.
    pub status_period_ms: u32,
    pub log_level: LogLevel,
}

/// Receive-only alert tuning, persisted on the device.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AlertSettings {
    /// Deauth frames per second that raise an alert, 1..=100.
    pub deauth_threshold: u8,
    /// Window the frames are counted over, 200..=5000 ms.
    pub deauth_window_ms: u16,
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
    /// Every persisted setting (reply to `GetSettingsAll`).
    SettingsAll {
        /// `SETTINGS_SCHEMA` of the firmware.
        schema: u8,
        /// How many times the settings were saved (proves persistence across reboots).
        revision: u32,
        scan: ScanSettings,
        radio: RadioSettings,
        telemetry: TelemetrySettings,
        alerts: AlertSettings,
    },
    /// Outcome of `SaveSettingsAll`. On `ok: false` nothing was written and `reason` says why.
    SettingsSaved {
        revision: u32,
        ok: bool,
        reason: Option<RejectReason>,
    },
    /// NVS usage and flash layout (reply to `GetStorageInfo`).
    StorageInfo {
        nvs_used: u32,
        nvs_free: u32,
        ota0_active: bool,
        ota1_active: bool,
        ota0_size: u32,
        ota1_size: u32,
        settings_revision: u32,
    },
    /// Outcome of `RunSelfTest`. `busy`: a scan is running, nothing was tested.
    SelfTestResult {
        radio: bool,
        nvs: bool,
        ble: bool,
        ota_slot: bool,
        busy: bool,
    },
    /// Firmware identity and build-time flags (reply to `GetDeviceInfo`).
    DeviceInfo {
        version: String<16>,
        git_sha: String<12>,
        built: String<24>,
        /// First 16 bytes of the SHA-256 of the embedded OTA public key; all zero when there is none.
        ota_key_fp: [u8; 16],
        request_conn_params: bool,
        coex_debug: bool,
        bonding: bool,
    },
    /// A request was refused outright (not a per-field settings error).
    Rejected {
        reason: RejectReason,
    },
    /// Whether the passive handshake listener is on (reply to `SetHandshakeCapture` / `GetHandshakeCapture`).
    HandshakeCaptureState {
        enabled: bool,
    },
    /// One EAPOL-Key frame of a WPA/WPA2 four-way handshake, heard passively (nothing was sent to cause it).
    HandshakeFrame {
        bssid: [u8; 6],
        client: [u8; 6],
        /// 1-4: which of the four handshake messages this is.
        message: u8,
        replay_counter: u64,
        /// True if the real frame was longer than `MAX_EAPOL_CAPTURE` and `eapol` holds only the start of it.
        truncated: bool,
        /// The raw EAPOL frame (from its own version byte), capped to `MAX_EAPOL_CAPTURE`.
        eapol: heapless::Vec<u8, MAX_EAPOL_CAPTURE>,
    },
    /// A BSSID/client pair now has enough messages (an ANonce-carrying one plus the SNonce+MIC one) for an
    /// offline cracking attempt, subject to the capture note on `HandshakeFrame.truncated`.
    HandshakeComplete {
        bssid: [u8; 6],
        client: [u8; 6],
    },
}
