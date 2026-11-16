/// Highest channel we allow (2.4 GHz, regulatory domain EU/JP style).
pub const MAX_CHANNEL: u8 = 13;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Idle,
    Scanning,
}
