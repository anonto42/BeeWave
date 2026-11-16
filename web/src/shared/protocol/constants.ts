/** Wire format version; must equal PROTO_VERSION in crates/protocol/src/types.rs. */
export const PROTO_VERSION = 1;

/** Largest OTA payload per chunk; must equal the Vec<u8, 96> capacity in Rust. */
export const MAX_OTA_CHUNK = 96;

/** Bytes before the postcard body: [version][sequence]. */
export const HEADER_LEN = 2;
