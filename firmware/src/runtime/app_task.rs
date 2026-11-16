use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use walpad_core::app::App;
use walpad_core::ports::{OtaWriter, RadioDriver, System, TelemetrySink};

use super::event::Event;

/// Status heartbeat period: also how long we wait for an event before ticking.
const TICK: Duration = Duration::from_secs(1);

/// The single task that owns `App`. All state lives here, so no locks are needed.
pub fn run_app<R, W, S, T>(mut app: App<R, W, S, T>, events: Receiver<Event>)
where
    R: RadioDriver,
    W: OtaWriter,
    S: System,
    T: TelemetrySink,
{
    loop {
        match events.recv_timeout(TICK) {
            Ok(Event::Command(c)) => app.on_command(c),
            Ok(Event::Frame(o)) => app.on_observation(o),
            Err(RecvTimeoutError::Timeout) => app.on_tick(),
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}
