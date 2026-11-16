use esp_idf_svc::sys::{esp_ota_handle_t, esp_partition_t};
use sha2::Sha256;

/// One update in flight.
pub(super) struct Session {
    pub handle: esp_ota_handle_t,
    pub partition: *const esp_partition_t,
    /// Hash of everything written so far; compared with the client's SHA-256 at the end.
    pub hasher: Sha256,
}
