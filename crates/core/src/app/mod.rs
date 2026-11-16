//! The application service: routes driving-adapter input to the feature slices.
mod keys;
mod messages;
mod service;

pub use keys::DeviceKeys;
pub use service::App;

#[cfg(test)]
mod tests;
