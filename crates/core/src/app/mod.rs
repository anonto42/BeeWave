//! The application service: routes driving-adapter input to the feature slices.
mod messages;
mod service;

pub use service::App;

#[cfg(test)]
mod tests;
