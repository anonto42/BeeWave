use beewave_protocol::{DeviceCommand, RejectReason};

use super::validate::validate;
use crate::ports::{Settings, SettingsError, SettingsStore};

pub struct SettingsService<S: SettingsStore> {
    pub(super) store: S,
    current: Settings,
}

impl<S: SettingsStore> SettingsService<S> {
    /// Loads whatever is saved (or defaults). A stored set that no longer validates (for example after
    /// a downgrade) is replaced by the defaults rather than trusted.
    pub fn new(mut store: S) -> Self {
        let loaded = store.load();
        let current = if validate(&loaded).is_ok() {
            loaded
        } else {
            Settings::default()
        };
        Self { store, current }
    }

    pub fn current(&self) -> &Settings {
        &self.current
    }

    /// How many times the settings were saved (persisted, survives reboots).
    pub fn revision(&mut self) -> u32 {
        self.store.save_revision()
    }

    /// Validate, persist, and only then adopt. On any error the stored and the live values are untouched.
    pub fn save_all(&mut self, next: Settings) -> Result<u32, RejectReason> {
        validate(&next)?;
        self.store.save(&next).map_err(|_| RejectReason::Storage)?;
        self.current = next;
        Ok(self.store.save_revision())
    }

    /// The legacy `SetSettings`: change only the pinned channel and scan-at-boot.
    pub fn set_legacy(&mut self, channel: u8, auto_scan: bool) -> Result<u32, RejectReason> {
        let mut next = self.current.clone();
        next.radio.default_channel = channel;
        next.scan.scan_at_boot = auto_scan;
        self.save_all(next)
    }

    /// `None` if not a (legacy) settings read. Writes go through `save_all` / `set_legacy`.
    pub fn handle_legacy_get(&self, cmd: &DeviceCommand) -> Option<(u8, bool)> {
        matches!(cmd, DeviceCommand::GetSettings).then_some((
            self.current.radio.default_channel,
            self.current.scan.scan_at_boot,
        ))
    }

    /// Restore factory defaults and erase everything stored (settings, key, revision).
    pub fn factory_reset(&mut self) -> Result<(), SettingsError> {
        self.store.erase_all()?;
        self.current = Settings::default();
        Ok(())
    }

    /// The device key stored by an earlier provisioning, if any.
    pub fn load_key(&mut self) -> Option<[u8; 32]> {
        self.store.load_key()
    }

    pub fn save_key(&mut self, key: &[u8; 32]) -> Result<(), SettingsError> {
        self.store.save_key(key)
    }
}
