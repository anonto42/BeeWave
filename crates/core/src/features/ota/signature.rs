use ed25519_dalek::{Signature, VerifyingKey};

/// Check an Ed25519 signature over an image's SHA-256 digest.
///
/// Uses strict verification (rejects malleable signatures and weak keys). A malformed public
/// key simply fails verification.
pub(super) fn verify(public_key: &[u8; 32], digest: &[u8; 32], r: &[u8; 32], s: &[u8; 32]) -> bool {
    let Ok(key) = VerifyingKey::from_bytes(public_key) else {
        return false;
    };
    let mut bytes = [0u8; 64];
    bytes[..32].copy_from_slice(r);
    bytes[32..].copy_from_slice(s);
    key.verify_strict(digest, &Signature::from_bytes(&bytes))
        .is_ok()
}
