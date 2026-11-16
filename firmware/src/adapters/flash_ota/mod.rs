//! Flash OTA adapter: implements the core's `OtaWriter` port with the ESP-IDF OTA API.
//!
//! Flow: `begin` picks the inactive slot (ota_0 / ota_1) -> `write` streams chunks to flash while
//! hashing them -> `finish` checks SHA-256, validates the image and sets it as the next boot
//! partition. The client then sends `RebootDevice`; after reboot `main` marks the image valid
//! (otherwise the bootloader rolls back to the previous slot).
mod session;
mod writer;

pub use writer::FlashOta;
