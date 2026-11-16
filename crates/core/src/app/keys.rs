/// Keys baked into the firmware at build time. A missing key fails closed: with no `auth` key no
/// client can authenticate, and with no `ota_signing` key every firmware update is refused.
#[derive(Clone, Copy, Default, Debug)]
pub struct DeviceKeys {
    /// Shared secret for the command handshake (HMAC-SHA256).
    pub auth: Option<[u8; 32]>,
    /// Ed25519 public key that firmware updates must be signed with.
    pub ota_signing: Option<[u8; 32]>,
}

/// Parse a 32-byte key written as exactly 64 hex characters (either case, surrounding whitespace
/// ignored). `None` for anything else, so a typo can never become a weak or partial key.
pub fn parse_hex_key(text: &str) -> Option<[u8; 32]> {
    let hex = text.trim().as_bytes();
    if hex.len() != 64 {
        return None;
    }
    let mut key = [0u8; 32];
    for (byte, pair) in key.iter_mut().zip(hex.chunks_exact(2)) {
        let hi = (pair[0] as char).to_digit(16)?;
        let lo = (pair[1] as char).to_digit(16)?;
        *byte = (hi * 16 + lo) as u8;
    }
    Some(key)
}
