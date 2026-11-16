use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use beewave_core::app::App;
use beewave_core::features::scan::Observation;
use beewave_core::ports::{OtaWriter, RadioDriver, SettingsStore, System, TelemetrySink};

use super::event::Event;

/// How long to wait for a command before servicing frames and the heartbeat.
const POLL: Duration = Duration::from_millis(10);
/// Status heartbeat period.
const TICK: Duration = Duration::from_secs(1);
/// Frames handled per loop turn, so a flood cannot delay the next command for long.
const FRAME_BATCH: usize = 32;

/// The single task that owns `App`. All state lives here, so no locks are needed.
/// Commands are served first; sniffed frames and the heartbeat fill the gaps.
pub fn run_app<R, W, S, T, P>(
    mut app: App<R, W, S, T, P>,
    events: Receiver<Event>,
    frames: Receiver<Observation>,
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
        let now = Instant::now();
        if now >= next_tick {
            app.on_tick();
            next_tick = now + TICK;
        }
    }
}
