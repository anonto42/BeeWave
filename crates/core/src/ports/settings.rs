use super::radio::AUTO_CHANNEL;

/// User settings kept in flash across reboots.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Settings {
    /// 0 = auto-hop, 1..=13 = fixed channel.
    pub channel: u8,
    /// Start scanning automatically at boot.
    pub auto_scan: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            channel: AUTO_CHANNEL,
            auto_scan: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SettingsError {
    InvalidChannel,
    Storage,
}

/// Persistent storage for `Settings` (NVS flash on the device).
pub trait SettingsStore {
    /// Saved settings, or defaults if nothing valid is stored.
    fn load(&mut self) -> Settings;
    fn save(&mut self, settings: &Settings) -> Result<(), SettingsError>;
}
