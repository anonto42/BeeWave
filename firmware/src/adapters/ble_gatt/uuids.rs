//! GATT UUIDs; must equal the constants in web/src/shared/transport/constants.ts.
use esp32_nimble::{uuid128, utilities::BleUuid};

pub const SERVICE: BleUuid = uuid128!("7a1c0001-9d3e-4b8a-a1f0-5e5b1c0de001");
pub const COMMAND: BleUuid = uuid128!("7a1c0002-9d3e-4b8a-a1f0-5e5b1c0de001");
pub const TELEMETRY: BleUuid = uuid128!("7a1c0003-9d3e-4b8a-a1f0-5e5b1c0de001");

/// Advertised name; the PWA filters on this prefix.
pub const DEVICE_NAME: &str = "WalPad";
