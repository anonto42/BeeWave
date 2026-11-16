use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

use walpad_protocol::TelemetryMessage;

use super::event::Event;

/// Queue depth. Bounded on purpose: a stalled consumer must not be able to eat the heap.
const QUEUE_DEPTH: usize = 32;

/// Both queues of the system:
/// - `events_*`: adapters -> app task (commands, frames)
/// - `tele_*`:   app task -> BLE (telemetry)
pub struct Channels {
    pub events_tx: SyncSender<Event>,
    pub events_rx: Receiver<Event>,
    pub tele_tx: SyncSender<TelemetryMessage>,
    pub tele_rx: Receiver<TelemetryMessage>,
}

pub fn channels() -> Channels {
    let (events_tx, events_rx) = sync_channel(QUEUE_DEPTH);
    let (tele_tx, tele_rx) = sync_channel(QUEUE_DEPTH);
    Channels { events_tx, events_rx, tele_tx, tele_rx }
}
