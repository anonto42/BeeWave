//! Build-time configuration, read from environment variables at compile time (64 hex chars each):
//! - `BEEWAVE_KEY` (optional): shared secret for the command handshake. Without it the device is
//!   *unclaimed*: the first client to connect provisions a key, which is stored in NVS.
//! - `BEEWAVE_OTA_PUBKEY`: Ed25519 public key firmware updates must be signed with.
//!
//! A missing `BEEWAVE_OTA_PUBKEY` fails closed: no update is accepted. A malformed `BEEWAVE_KEY`
//! is ignored (logged), which also leaves the device unclaimed.
use beewave_core::app::{parse_hex_key, DeviceKeys};

/// All device keys, logging loudly about any that are missing.
pub fn device_keys() -> DeviceKeys {
    DeviceKeys {
        auth: auth_key(option_env!("BEEWAVE_KEY")),
        ota_signing: key(
            "BEEWAVE_OTA_PUBKEY",
            option_env!("BEEWAVE_OTA_PUBKEY"),
            "firmware updates are DISABLED",
        ),
    }
}

/// The optional build-time device key. Absent is normal (the device claims itself on first connect);
/// present but malformed is logged because it was probably meant to be used.
fn auth_key(value: Option<&str>) -> Option<[u8; 32]> {
    match value {
        None => {
            log::info!("no build-time BEEWAVE_KEY: using a stored key, or waiting to be claimed");
            None
        }
        Some(v) => {
            let parsed = parse_hex_key(v);
            if parsed.is_none() {
                log::error!("BEEWAVE_KEY is not 64 hex characters; ignoring it");
            }
            parsed
        }
    }
}

fn key(name: &str, value: Option<&str>, consequence: &str) -> Option<[u8; 32]> {
    let parsed = value.and_then(parse_hex_key);
    if parsed.is_none() {
        log::error!("{name} missing or not 64 hex characters; {consequence}");
    }
    parsed
}
