//! Ports: the traits the core needs from the outside world.
//!
//! A *trait* is a list of methods a type promises to have (like an interface).
//! The core is written against these traits; `firmware/src/adapters` provides the real
//! implementations and the tests provide fakes. One file per port.
mod ota;
mod radio;
mod settings;
mod system;
mod telemetry;

pub use ota::{OtaError, OtaWriter};
pub use radio::{RadioDriver, RadioError, AUTO_CHANNEL, MAX_CHANNEL};
pub use settings::{Settings, SettingsError, SettingsStore};
pub use system::{BuildFacts, PartitionInfo, System};
pub use telemetry::TelemetrySink;
