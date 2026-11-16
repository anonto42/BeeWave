//! Persistent settings in NVS flash (the `nvs` partition). Implements the core's `SettingsStore` port.
mod store;

pub use store::NvsSettings;
