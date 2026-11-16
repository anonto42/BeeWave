use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use esp_idf_svc::sys::EspError;
use walpad_core::ports::{Settings, SettingsError, SettingsStore, AUTO_CHANNEL, MAX_CHANNEL};

/// NVS namespace and keys (keys are limited to 15 characters).
const NAMESPACE: &str = "walpad";
const KEY_CHANNEL: &str = "channel";
const KEY_AUTO_SCAN: &str = "autoscan";

pub struct NvsSettings {
    nvs: EspNvs<NvsDefault>,
}

impl NvsSettings {
    pub fn new(partition: EspDefaultNvsPartition) -> Result<Self, EspError> {
        Ok(Self { nvs: EspNvs::new(partition, NAMESPACE, true)? })
    }
}

impl SettingsStore for NvsSettings {
    fn load(&mut self) -> Settings {
        let defaults = Settings::default();
        // Anything missing, unreadable or out of range falls back to the default.
        let channel = self
            .nvs
            .get_u8(KEY_CHANNEL)
            .ok()
            .flatten()
            .filter(|c| *c == AUTO_CHANNEL || (1..=MAX_CHANNEL).contains(c))
            .unwrap_or(defaults.channel);
        let auto_scan = self
            .nvs
            .get_u8(KEY_AUTO_SCAN)
            .ok()
            .flatten()
            .map(|v| v != 0)
            .unwrap_or(defaults.auto_scan);
        Settings { channel, auto_scan }
    }

    fn save(&mut self, s: &Settings) -> Result<(), SettingsError> {
        self.nvs
            .set_u8(KEY_CHANNEL, s.channel)
            .and_then(|_| self.nvs.set_u8(KEY_AUTO_SCAN, s.auto_scan as u8))
            .map_err(|_| SettingsError::Storage)
    }
}
