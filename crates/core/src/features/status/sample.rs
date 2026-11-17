use beewave_protocol::TelemetryMessage;

use crate::ports::System;

/// Build a `Status` message from the device facts behind the `System` port.
pub fn sample<S: System>(sys: &S, channel: u8, dropped: u16, scanning: bool) -> TelemetryMessage {
    TelemetryMessage::Status {
        active_channel: channel,
        free_heap: sys.free_heap(),
        cpu_load: sys.cpu_load(),
        dropped,
        scanning,
    }
}
