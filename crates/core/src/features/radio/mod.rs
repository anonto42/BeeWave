//! Radio control: scan on/off and channel selection with validation.
mod controller;
mod hop;
mod types;

pub use controller::RadioController;
pub use hop::next_in_order;
pub use types::{Mode, AUTO_CHANNEL, MAX_CHANNEL};

#[cfg(test)]
mod tests;
