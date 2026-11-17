use beewave_protocol::{
    AlertSettings, HopOrder, LogLevel, RadioSettings, ScanSettings, TelemetrySettings, HOP_LIST_LEN,
};

use super::radio::AUTO_CHANNEL;

/// Everything the owner can persist on the device, in the four groups of the wire protocol.
/// Ranges and cross-field rules are checked by `features::settings::validate`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Settings {
    pub scan: ScanSettings,
    pub radio: RadioSettings,
    pub telemetry: TelemetrySettings,
    pub alerts: AlertSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            scan: ScanSettings {
                scan_at_boot: false,
                scan_interval_ms: 250,
                dedup_rssi_db: 6,
                ssid_filter: heapless::String::new(),
                channel_filter_mask: 0,
            },
            radio: RadioSettings {
                default_channel: AUTO_CHANNEL,
                hop_dwell_ms: 250,
                hop_order: HopOrder::Ascending,
                custom_hop_list: [0; HOP_LIST_LEN],
            },
            telemetry: TelemetrySettings {
                status_period_ms: 1000,
                log_level: LogLevel::Info,
            },
            alerts: AlertSettings {
                deauth_threshold: 10,
                deauth_window_ms: 1000,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SettingsError {
    Storage,
}

/// Persistent storage for `Settings` and the claimed device key (NVS flash on the device).
pub trait SettingsStore {
    /// Saved settings, or defaults if nothing valid is stored.
    fn load(&mut self) -> Settings;
    /// Persist all settings and bump the revision counter. A failed save leaves the old values.
    fn save(&mut self, settings: &Settings) -> Result<(), SettingsError>;
    /// How many times `save` succeeded since the last erase (survives reboots).
    fn save_revision(&mut self) -> u32;
    /// The device key a client provisioned, if any (persisted across reboots).
    fn load_key(&mut self) -> Option<[u8; 32]>;
    fn save_key(&mut self, key: &[u8; 32]) -> Result<(), SettingsError>;
    /// Remove everything this store holds: settings, key and revision (factory reset).
    fn erase_all(&mut self) -> Result<(), SettingsError>;
}
