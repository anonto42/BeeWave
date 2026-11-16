//! OTA: a strictly sequential, acknowledged, resumable-by-offset update state machine.
mod session;
mod state;

pub use session::OtaSession;

#[cfg(test)]
mod tests;
