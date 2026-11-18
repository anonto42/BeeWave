use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

use beewave_core::features::handshake::EapolFrame;
use beewave_core::features::probe::ProbeObservation;
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
/// Wi-Fi sniffer -> app task (classified EAPOL frames; a handshake is a handful of frames, lossy when full).
pub const HANDSHAKE_DEPTH: usize = 16;
/// Wi-Fi sniffer -> app task (probe request frames; lossy when full).
pub const PROBE_DEPTH: usize = 32;
/// App task -> BLE (telemetry; lossy when full).
pub const TELEMETRY_DEPTH: usize = 32;
/// Wi-Fi sniffer -> app task (observations; lossy when full).
pub const FRAME_DEPTH: usize = 64;

/// All queues of the system:
/// - `events_*`: BLE -> app task (commands)
/// - `frames_*`: Wi-Fi sniffer -> app task (lossy under load)
/// - `ble_*`:    passive BLE observer -> app task (lossy under load)
/// - `handshake_*`: Wi-Fi sniffer -> app task, classified EAPOL frames (lossy under load)
/// - `tele_*`:   app task -> BLE (telemetry, lossy)
pub struct Channels {
    pub events_tx: SyncSender<Event>,
    pub events_rx: Receiver<Event>,
    pub frames_tx: SyncSender<Observation>,
    pub frames_rx: Receiver<Observation>,
    pub ble_tx: SyncSender<AdClass>,
    pub ble_rx: Receiver<AdClass>,
    pub handshake_tx: SyncSender<EapolFrame>,
    pub handshake_rx: Receiver<EapolFrame>,
    pub probes_tx: SyncSender<ProbeObservation>,
    pub probes_rx: Receiver<ProbeObservation>,
    pub tele_tx: SyncSender<Timed<TelemetryMessage>>,
    pub tele_rx: Receiver<Timed<TelemetryMessage>>,
}

pub fn channels() -> Channels {
    let (events_tx, events_rx) = sync_channel(EVENT_DEPTH);
    let (frames_tx, frames_rx) = sync_channel(FRAME_DEPTH);
    let (ble_tx, ble_rx) = sync_channel(BLE_ADV_DEPTH);
    let (handshake_tx, handshake_rx) = sync_channel(HANDSHAKE_DEPTH);
    let (probes_tx, probes_rx) = sync_channel(PROBE_DEPTH);
    let (tele_tx, tele_rx) = sync_channel(TELEMETRY_DEPTH);
    Channels {
        events_tx,
        events_rx,
        frames_tx,
        frames_rx,
        ble_tx,
        ble_rx,
        handshake_tx,
        handshake_rx,
        probes_tx,
        probes_rx,
        tele_tx,
        tele_rx,
    }
}
