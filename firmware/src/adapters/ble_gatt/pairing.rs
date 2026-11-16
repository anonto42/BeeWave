//! Pairing passkey. The phone shows a "pairing request" and the user types this 6-digit code.
//! Set your own at build time:  WALPAD_PASSKEY=482913 make fw-build
//! (the default below is public, so change it before using the device anywhere shared).

pub const DEFAULT_PASSKEY: u32 = 123_456;

pub fn passkey() -> u32 {
    match option_env!("WALPAD_PASSKEY").and_then(|s| s.parse::<u32>().ok()).filter(|p| *p <= 999_999) {
        Some(p) => p,
        None => {
            log::warn!("WALPAD_PASSKEY not set; using the public default passkey");
            DEFAULT_PASSKEY
        }
    }
}
