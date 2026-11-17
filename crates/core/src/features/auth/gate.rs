use beewave_protocol::{TelemetryMessage, AUTH_CONTEXT, AUTH_NONCE_LEN};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Per-connection session state. The caller supplies the random nonce so this stays pure.
///
/// With no key the gate is *unclaimed*: every proof fails until a client claims it with `claim`.
pub struct AuthGate {
    key: Option<[u8; 32]>,
    nonce: Option<[u8; AUTH_NONCE_LEN]>,
    authenticated: bool,
}

impl AuthGate {
    pub const fn new(key: Option<[u8; 32]>) -> Self {
        Self {
            key,
            nonce: None,
            authenticated: false,
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    /// Does the device have a key (built in, stored or provisioned)?
    pub fn is_claimed(&self) -> bool {
        self.key.is_some()
    }

    /// First-time provisioning: succeeds only while there is no key, and refuses the all-zero key.
    /// The claiming client is authenticated for this session (it just chose the key).
    pub fn claim(&mut self, key: [u8; 32]) -> bool {
        if self.key.is_some() || key == [0u8; 32] {
            return false;
        }
        self.key = Some(key);
        self.nonce = None;
        self.authenticated = true;
        true
    }

    /// Start a handshake with a fresh nonce (replaces any pending one).
    pub fn begin(&mut self, nonce: [u8; AUTH_NONCE_LEN]) -> TelemetryMessage {
        self.nonce = Some(nonce);
        TelemetryMessage::AuthChallenge { nonce }
    }

    /// Check a proof against the pending nonce. The nonce is consumed whatever the outcome,
    /// so a proof can never be replayed.
    pub fn prove(&mut self, mac: &[u8; 32]) -> TelemetryMessage {
        let pending = self.nonce.take();
        let ok = match (&self.key, pending) {
            (Some(key), Some(nonce)) => verify(key, &nonce, mac),
            _ => false,
        };
        if ok {
            self.authenticated = true;
        }
        TelemetryMessage::AuthResult { ok }
    }

    /// End the session (client connected or disconnected).
    pub fn reset(&mut self) {
        self.nonce = None;
        self.authenticated = false;
    }
}

/// The proof the client must send: `HMAC-SHA256(key, AUTH_CONTEXT || nonce)`.
pub fn proof(key: &[u8; 32], nonce: &[u8; AUTH_NONCE_LEN]) -> Option<[u8; 32]> {
    let mut mac = HmacSha256::new_from_slice(key).ok()?;
    mac.update(AUTH_CONTEXT);
    mac.update(nonce);
    Some(mac.finalize().into_bytes().into())
}

fn verify(key: &[u8; 32], nonce: &[u8; AUTH_NONCE_LEN], tag: &[u8; 32]) -> bool {
    let Ok(mut mac) = HmacSha256::new_from_slice(key) else {
        return false;
    };
    mac.update(AUTH_CONTEXT);
    mac.update(nonce);
    // `verify_slice` compares in constant time.
    mac.verify_slice(tag).is_ok()
}
