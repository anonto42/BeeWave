use beewave_protocol::DeviceCommand;

use crate::ports::{Settings, SettingsError, SettingsStore, AUTO_CHANNEL, MAX_CHANNEL};

pub struct SettingsService<S: SettingsStore> {
    store: S,
    current: Settings,
}

impl<S: SettingsStore> SettingsService<S> {
    /// Loads whatever is saved (or defaults).
    pub fn new(mut store: S) -> Self {
        let current = store.load();
        Self { store, current }
    }

    pub fn current(&self) -> Settings {
        self.current
    }

    /// The device key stored by an earlier provisioning, if any.
    pub fn load_key(&mut self) -> Option<[u8; 32]> {
        self.store.load_key()
    }

    pub fn save_key(&mut self, key: &[u8; 32]) -> Result<(), SettingsError> {
        self.store.save_key(key)
    }

    /// `None` if not a settings command. `Some(Ok(settings))` carries the settings to report back.
    pub fn handle(&mut self, cmd: &DeviceCommand) -> Option<Result<Settings, SettingsError>> {
        Some(match cmd {
            DeviceCommand::GetSettings => Ok(self.current),
            DeviceCommand::SetSettings { channel, auto_scan } => self.set(*channel, *auto_scan),
            _ => return None,
        })
    }

    fn set(&mut self, channel: u8, auto_scan: bool) -> Result<Settings, SettingsError> {
        if channel != AUTO_CHANNEL && !(1..=MAX_CHANNEL).contains(&channel) {
            return Err(SettingsError::InvalidChannel);
        }
        let next = Settings { channel, auto_scan };
        self.store.save(&next)?; // only update in memory if flash accepted it
        self.current = next;
        Ok(next)
    }
}
