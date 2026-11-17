use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use beewave_core::app::App;
use beewave_core::features::scan::Observation;
use beewave_core::ports::{OtaWriter, RadioDriver, SettingsStore, System, TelemetrySink};
use beewave_protocol::AdClass;

use super::event::Event;

/// How long to wait for a command before servicing frames and the heartbeat.
const POLL: Duration = Duration::from_millis(10);
/// Status heartbeat period.
const TICK: Duration = Duration::from_secs(1);
/// Per-second deauth counts: BSSIDs and how many frames each sent.
pub type DeauthCounts = [([u8; 6], u16); DEAUTH_SLOTS];
/// Size of `DeauthCounts` (the sniffer's counter table).
pub const DEAUTH_SLOTS: usize = 8;
/// Advertisements handled per loop turn.
const BLE_BATCH: usize = 32;
/// Frames handled per loop turn, so a flood cannot delay the next command for long.
const FRAME_BATCH: usize = 32;

/// The single task that owns `App`. All state lives here, so no locks are needed.
/// Commands are served first; sniffed frames and the heartbeat fill the gaps.
///
/// The two hooks keep this file free of adapter types: `drain_deauth` fills the per-second deauth counts
/// from the sniffer's counters and returns how many entries are valid; `set_ble_observer` mirrors the
/// owner's BLE-monitor switch into the observer thread.
pub fn run_app<R, W, S, T, P>(
    mut app: App<R, W, S, T, P>,
    events: Receiver<Event>,
    frames: Receiver<Observation>,
    ble: Receiver<AdClass>,
    drain_deauth: fn(&mut DeauthCounts) -> usize,
    set_ble_observer: fn(bool),
) where
    R: RadioDriver,
    W: OtaWriter,
    S: System,
    T: TelemetrySink,
    P: SettingsStore,
{
    app.start(); // apply saved channel / auto-scan

    let mut next_tick = Instant::now() + TICK;
    loop {
        match events.recv_timeout(POLL) {
            Ok(Event::Command(c)) => app.on_command(c),
            Ok(Event::LinkReset) => app.on_link_reset(),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        for _ in 0..FRAME_BATCH {
            match frames.try_recv() {
                Ok(o) => app.on_observation(o),
                Err(_) => break,
            }
        }
        for _ in 0..BLE_BATCH {
            match ble.try_recv() {
                Ok(class) => app.on_ble_adv(class),
                Err(_) => break,
            }
        }
        let now = Instant::now();
        if now >= next_tick {
            let mut counts: DeauthCounts = [([0; 6], 0); DEAUTH_SLOTS];
            let n = drain_deauth(&mut counts);
            if n > 0 {
                app.on_deauth_second(&counts[..n]);
            }
            set_ble_observer(app.ble_monitor_enabled());
            app.on_tick();
            next_tick = now + TICK;
        }
    }
}
