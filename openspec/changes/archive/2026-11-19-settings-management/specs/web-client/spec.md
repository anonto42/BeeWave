## ADDED Requirements

### Requirement: Settings management surface
The client SHALL present device-side settings in sections with one Apply and Revert bar and per-section dirty tracking, SHALL send one `SaveSettingsAll` on Apply and show the reason and highlight the field on rejection, SHALL save phone-side settings immediately, and SHALL load device settings only after authentication.

#### Scenario: Rejected save
- **WHEN** the device answers `SettingsSaved` with `ok: false`
- **THEN** the bar shows the reason, the field is highlighted and the confirmed values are unchanged

#### Scenario: Before authentication
- **WHEN** the link is up but authentication has not succeeded
- **THEN** no settings request is sent

### Requirement: Settings import and export
The client SHALL export phone-side settings (and optionally history) as JSON with a schema version, SHALL import such a file only after showing the differences, SHALL reject unknown keys, and SHALL export device settings as a separate file.

#### Scenario: Unknown key
- **WHEN** an imported file contains an unknown key
- **THEN** the import is refused and nothing changes
