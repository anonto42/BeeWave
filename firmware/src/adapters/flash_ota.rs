//! Flash OTA adapter (OtaWriter port). TODO: esp_idf_svc::ota::EspOta, SHA-256 via mbedtls, set boot partition.
use walpad_core::ports::{OtaError, OtaWriter};

pub struct FlashOta;

impl OtaWriter for FlashOta {
    fn begin(&mut self, _size: u32) -> Result<(), OtaError> { Err(OtaError::Unsupported) }
    fn write(&mut self, _data: &[u8]) -> Result<(), OtaError> { Err(OtaError::Unsupported) }
    fn finish(&mut self, _sha: &[u8; 32]) -> Result<(), OtaError> { Err(OtaError::Unsupported) }
    fn abort(&mut self) {}
}
