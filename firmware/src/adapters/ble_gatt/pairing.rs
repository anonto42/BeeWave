//! Passkey pairing: written and working at the BLE/NimBLE level, but **not wired in**
//! (`server.rs` does not call `require()`). Reason it's off: Android's own "pair new device"
//! screen did not reliably bond with this plain custom GATT peripheral, and Web Bluetooth has no
//! way to drive that pairing UI either, so an encrypted characteristic just hung with no visible
//! error on either side. Revisit once there's a real need for it (e.g. before this device is ever
//! reachable by anyone other than you): call `require(device)` where the device is created, and
//! put `WRITE_ENC | WRITE_AUTHEN` back on the command characteristic in `server.rs`.
//!
//! Set your own passkey at build time:  BEEWAVE_PASSKEY=482913 make fw-build
//! (the default below is public; change it before relying on this for real security).
use esp32_nimble::enums::{AuthReq, SecurityIOCap};
use esp32_nimble::BLEDevice;

pub const DEFAULT_PASSKEY: u32 = 123_456;

#[allow(dead_code)]
pub fn require(device: &mut BLEDevice) {
    device
        .security()
        .set_auth(AuthReq::Bond | AuthReq::Mitm | AuthReq::Sc)
        .set_io_cap(SecurityIOCap::DisplayOnly)
        .set_passkey(passkey());
}

fn passkey() -> u32 {
    match option_env!("BEEWAVE_PASSKEY")
        .and_then(|s| s.parse::<u32>().ok())
        .filter(|p| *p <= 999_999)
    {
        Some(p) => p,
        None => {
            log::warn!("BEEWAVE_PASSKEY not set; using the public default passkey");
            DEFAULT_PASSKEY
        }
    }
}
