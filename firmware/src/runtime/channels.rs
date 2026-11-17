use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

use beewave_core::features::scan::Observation;
use beewave_protocol::{AdClass, TelemetryMessage};

use super::event::Event;
use super::timed::Timed;

// ---- Tuning knobs (queue depths) -------------------------------------------------------------
// Bounded on purpose: a stalled consumer must not be able to eat the heap. Each slot costs RAM
// (telemetry slots hold a message plus a timestamp), so raise these only with measurements.
/// BLE -> app task (commands and link resets).
pub const EVENT_DEPTH: usize = 16;
/// BLE observer -> app task (classified advertisements; lossy when full).
pub const BLE_ADV_DEPTH: usize = 64;
/// App task -> BLE (telemetry; lossy when full).
pub const TELEMETRY_DEPTH: usize = 32;
/// Wi-Fi sniffer -> app task (observations; lossy when full).
pub const FRAME_DEPTH: usize = 64;

/// All queues of the system:
/// - `events_*`: BLE -> app task (commands)
/// - `frames_*`: Wi-Fi sniffer -> app task (lossy under load)
/// - `ble_*`:    passive BLE observer -> app task (lossy under load)
/// - `tele_*`:   app task -> BLE (telemetry, lossy)
pub struct Channels {
    pub events_tx: SyncSender<Event>,
    pub events_rx: Receiver<Event>,
    pub frames_tx: SyncSender<Observation>,
    pub frames_rx: Receiver<Observation>,
    pub ble_tx: SyncSender<AdClass>,
    pub ble_rx: Receiver<AdClass>,
    pub tele_tx: SyncSender<Timed<TelemetryMessage>>,
    pub tele_rx: Receiver<Timed<TelemetryMessage>>,
}

pub fn channels() -> Channels {
    let (events_tx, events_rx) = sync_channel(EVENT_DEPTH);
    let (frames_tx, frames_rx) = sync_channel(FRAME_DEPTH);
    let (ble_tx, ble_rx) = sync_channel(BLE_ADV_DEPTH);
    let (tele_tx, tele_rx) = sync_channel(TELEMETRY_DEPTH);
    Channels {
        events_tx,
        events_rx,
        frames_tx,
        frames_rx,
        ble_tx,
        ble_rx,
        tele_tx,
        tele_rx,
    }
}
