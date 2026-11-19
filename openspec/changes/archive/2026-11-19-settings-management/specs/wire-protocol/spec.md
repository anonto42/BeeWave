## ADDED Requirements

### Requirement: Settings management messages
The protocol SHALL define `GetSettingsAll`, `SaveSettingsAll`, `FactoryReset`, `RunSelfTest`, `GetStorageInfo` and `GetDeviceInfo` commands and `SettingsAll`, `SettingsSaved`, `StorageInfo`, `SelfTestResult`, `DeviceInfo` and `Rejected` telemetry, appended after existing variants with the version unchanged, and SHALL keep `GetSettings` and `SetSettings`.

#### Scenario: Golden vectors
- **WHEN** Rust encodes each settings message
- **THEN** the TypeScript codec produces and decodes identical bytes, checked by golden vectors
