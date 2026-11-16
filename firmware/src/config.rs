//! Build-time configuration. The device authentication key comes from `BEEWAVE_KEY`
//! (64 hex characters). Anything else leaves the device locked (no client can authenticate).

/// Parse the compile-time `BEEWAVE_KEY`; `None` (locked) if it is unset or malformed.
pub fn auth_key() -> Option<[u8; 32]> {
    let key = parse_key(option_env!("BEEWAVE_KEY")?);
    if key.is_none() {
        log::error!("BEEWAVE_KEY must be 64 hex characters; device is LOCKED");
    }
    key
}

fn parse_key(hex: &str) -> Option<[u8; 32]> {
    let hex = hex.trim().as_bytes();
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
