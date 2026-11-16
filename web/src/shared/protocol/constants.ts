/** Wire format version; must equal PROTO_VERSION in crates/protocol/src/types.rs. */
export const PROTO_VERSION = 1;

/** Largest OTA payload per chunk; must equal the Vec<u8, 96> capacity in Rust. */
export const MAX_OTA_CHUNK = 96;

/** Ed25519 signature length (R||S) sent with `otaEndSigned`. */
export const SIGNATURE_LEN = 64;

/** Nonce length of the auth challenge; must equal AUTH_NONCE_LEN in Rust. */
export const AUTH_NONCE_LEN = 16;

/** Domain separator signed with the nonce; must equal AUTH_CONTEXT in crates/protocol. */
export const AUTH_CONTEXT = "beewave-auth-v1";

/** Bytes before the postcard body: [version][sequence]. */
export const HEADER_LEN = 2;
