//! Radio types. The channel constants live in the `ports::radio` contract and are re-exported here.
pub use crate::ports::{AUTO_CHANNEL, MAX_CHANNEL};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Idle,
    Scanning,
}
