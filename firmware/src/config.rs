//! Build-time configuration, read from environment variables at compile time (64 hex chars each):
//! - `BEEWAVE_KEY`: shared secret for the command handshake.
//! - `BEEWAVE_OTA_PUBKEY`: Ed25519 public key firmware updates must be signed with.
//!
//! A missing or malformed value fails closed: no client can authenticate / no update is accepted.
use beewave_core::app::{parse_hex_key, DeviceKeys};

/// All device keys, logging loudly about any that are missing.
pub fn device_keys() -> DeviceKeys {
    DeviceKeys {
        auth: key(
            "BEEWAVE_KEY",
            option_env!("BEEWAVE_KEY"),
            "device is LOCKED",
        ),
        ota_signing: key(
            "BEEWAVE_OTA_PUBKEY",
            option_env!("BEEWAVE_OTA_PUBKEY"),
            "firmware updates are DISABLED",
        ),
    }
}

fn key(name: &str, value: Option<&str>, consequence: &str) -> Option<[u8; 32]> {
    let parsed = value.and_then(parse_hex_key);
    if parsed.is_none() {
        log::error!("{name} missing or not 64 hex characters; {consequence}");
    }
    parsed
}
