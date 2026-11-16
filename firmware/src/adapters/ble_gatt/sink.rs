use std::sync::mpsc::{SyncSender, TrySendError};

use beewave_core::ports::TelemetrySink;
use beewave_protocol::TelemetryMessage;

/// The `TelemetrySink` handed to the core. Non-blocking and lossy by design:
/// a slow BLE link must never stall the app task.
pub struct BleSink(pub SyncSender<TelemetryMessage>);

impl TelemetrySink for BleSink {
    fn publish(&mut self, msg: TelemetryMessage) -> bool {
        !matches!(
            self.0.try_send(msg),
            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_))
        )
    }
}
