/// Keys baked into the firmware at build time. A missing key fails closed: with no `auth` key no
/// client can authenticate, and with no `ota_signing` key every firmware update is refused.
#[derive(Clone, Copy, Default, Debug)]
pub struct DeviceKeys {
    /// Shared secret for the command handshake (HMAC-SHA256).
    pub auth: Option<[u8; 32]>,
    /// Ed25519 public key that firmware updates must be signed with.
    pub ota_signing: Option<[u8; 32]>,
}
