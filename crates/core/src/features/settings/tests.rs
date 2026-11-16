use beewave_protocol::DeviceCommand;

use super::*;
use crate::ports::{Settings, SettingsError, SettingsStore};

#[derive(Default)]
struct FakeStore {
    saved: Option<Settings>,
    fail: bool,
}

impl SettingsStore for FakeStore {
    fn load(&mut self) -> Settings {
        self.saved.unwrap_or_default()
    }
    fn save(&mut self, s: &Settings) -> Result<(), SettingsError> {
        if self.fail {
            return Err(SettingsError::Storage);
        }
        self.saved = Some(*s);
        Ok(())
    }
}

#[test]
fn loads_saved_settings_at_start() {
    let store = FakeStore {
        saved: Some(Settings {
            channel: 6,
            auto_scan: true,
        }),
        fail: false,
    };
    let s = SettingsService::new(store);
    assert_eq!(
        s.current(),
        Settings {
            channel: 6,
            auto_scan: true
        }
    );
}

#[test]
fn set_validates_saves_and_reports() {
    let mut s = SettingsService::new(FakeStore::default());
    assert_eq!(
        s.handle(&DeviceCommand::SetSettings {
            channel: 99,
            auto_scan: false
        }),
        Some(Err(SettingsError::InvalidChannel))
    );
    assert_eq!(
        s.handle(&DeviceCommand::SetSettings {
            channel: 11,
            auto_scan: true
        }),
        Some(Ok(Settings {
            channel: 11,
            auto_scan: true
        }))
    );
    assert_eq!(
        s.handle(&DeviceCommand::GetSettings),
        Some(Ok(Settings {
            channel: 11,
            auto_scan: true
        }))
    );
    assert_eq!(s.handle(&DeviceCommand::RebootDevice), None);
}

#[test]
fn storage_failure_keeps_old_settings() {
    let mut s = SettingsService::new(FakeStore {
        fail: true,
        ..Default::default()
    });
    assert_eq!(
        s.handle(&DeviceCommand::SetSettings {
            channel: 3,
            auto_scan: true
        }),
        Some(Err(SettingsError::Storage))
    );
    assert_eq!(s.current(), Settings::default());
}
