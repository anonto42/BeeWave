/// Highest channel we allow (2.4 GHz, regulatory domain EU/JP style).
pub const MAX_CHANNEL: u8 = 13;

/// `ConfigureRadio { channel: 0 }` means "hop across all channels".
pub const AUTO_CHANNEL: u8 = 0;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Idle,
    Scanning,
}
