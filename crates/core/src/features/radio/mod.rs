//! Radio control: scan on/off and channel selection with validation.
mod controller;
mod types;

pub use controller::RadioController;
pub use types::{Mode, AUTO_CHANNEL, MAX_CHANNEL};

#[cfg(test)]
mod tests;
