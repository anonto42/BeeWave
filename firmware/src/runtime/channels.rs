use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

use beewave_core::features::scan::Observation;
use beewave_protocol::TelemetryMessage;

use super::event::Event;

/// Queue depths. Bounded on purpose: a stalled consumer must not be able to eat the heap.
const EVENT_DEPTH: usize = 16;
const TELEMETRY_DEPTH: usize = 32;
const FRAME_DEPTH: usize = 64;

/// All queues of the system:
/// - `events_*`: BLE -> app task (commands)
/// - `frames_*`: Wi-Fi sniffer -> app task (lossy under load)
/// - `tele_*`:   app task -> BLE (telemetry, lossy)
pub struct Channels {
    pub events_tx: SyncSender<Event>,
    pub events_rx: Receiver<Event>,
    pub frames_tx: SyncSender<Observation>,
    pub frames_rx: Receiver<Observation>,
    pub tele_tx: SyncSender<TelemetryMessage>,
    pub tele_rx: Receiver<TelemetryMessage>,
}

pub fn channels() -> Channels {
    let (events_tx, events_rx) = sync_channel(EVENT_DEPTH);
    let (frames_tx, frames_rx) = sync_channel(FRAME_DEPTH);
    let (tele_tx, tele_rx) = sync_channel(TELEMETRY_DEPTH);
    Channels {
        events_tx,
        events_rx,
        frames_tx,
        frames_rx,
        tele_tx,
        tele_rx,
    }
}
