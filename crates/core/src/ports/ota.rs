/// Writer for the inactive firmware slot.
pub trait OtaWriter {
    fn begin(&mut self, size: u32) -> Result<(), OtaError>;
    fn write(&mut self, data: &[u8]) -> Result<(), OtaError>;
    /// Verify SHA-256 of everything written, then mark the slot bootable.
    fn finish(&mut self, expected_sha256: &[u8; 32]) -> Result<(), OtaError>;
    fn abort(&mut self);
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OtaError {
    Unsupported,
    TooLarge,
    BadOffset,
    BadState,
    HashMismatch,
    Flash,
}
