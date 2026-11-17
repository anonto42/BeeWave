//! The application service: routes driving-adapter input to the feature slices.
mod keys;
mod messages;
mod monitoring;
mod service;

pub use keys::{parse_hex_key, DeviceKeys};
pub use monitoring::{InvalidMonitorConfig, MonitorConfig, Monitoring, FACTOR_RANGE};
pub use service::App;

#[cfg(test)]
mod key_tests;
#[cfg(test)]
mod tests;
