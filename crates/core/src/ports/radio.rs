/// Highest channel we allow (2.4 GHz, regulatory domain EU/JP style).
pub const MAX_CHANNEL: u8 = 13;

/// Channel value meaning "hop across all channels while scanning".
pub const AUTO_CHANNEL: u8 = 0;

/// The 2.4 GHz radio.
pub trait RadioDriver {
    /// Pin to `channel` (1..=MAX_CHANNEL), or pass 0 (`AUTO_CHANNEL`) to hop across all channels while scanning.
    fn set_channel(&mut self, channel: u8) -> Result<(), RadioError>;
    fn start_sniffing(&mut self) -> Result<(), RadioError>;
    fn stop_sniffing(&mut self) -> Result<(), RadioError>;
    /// The channel the radio is on right now (changes while hopping).
    fn channel(&self) -> u8;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RadioError {
    InvalidChannel,
    Driver,
}
