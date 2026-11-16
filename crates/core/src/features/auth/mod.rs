//! Authentication: a challenge-response gate that decides whether commands are accepted.
mod gate;

pub use gate::{proof, AuthGate};

#[cfg(test)]
mod tests;
