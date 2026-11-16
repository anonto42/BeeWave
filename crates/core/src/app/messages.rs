//! Event telemetry the app itself raises (as opposed to feature output).
use beewave_protocol::TelemetryMessage;
use heapless::String;

use crate::ports::RadioError;

/// Codes carried in `TelemetryMessage::Event`.
pub const EVENT_RADIO_ERROR: u16 = 1;
pub const EVENT_OTA_ERROR: u16 = 2;
pub const EVENT_SETTINGS_ERROR: u16 = 3;

pub fn event(code: u16, text: &str) -> TelemetryMessage {
    let mut message = String::new();
    // push_str fails if the text exceeds capacity; a truncated message is fine for an event.
    let _ = message.push_str(&text[..text.len().min(64)]);
    TelemetryMessage::Event { code, message }
}

pub fn radio_error_text(e: RadioError) -> &'static str {
    match e {
        RadioError::InvalidChannel => "invalid channel",
        RadioError::Driver => "radio driver error",
    }
}
