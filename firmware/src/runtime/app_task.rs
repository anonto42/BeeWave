use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use beewave_core::app::App;
use beewave_core::features::handshake::EapolFrame;
use beewave_core::features::probe::ProbeObservation;
use beewave_core::features::scan::Observation;
use beewave_core::ports::{OtaWriter, RadioDriver, SettingsStore, System, TelemetrySink};
use beewave_protocol::{AdClass, LogLevel};

use super::event::Event;

/// How long to wait for a command before servicing frames and the heartbeat.
const POLL: Duration = Duration::from_millis(10);
/// Deauth counts over one window: BSSIDs and how many frames each sent.
pub type DeauthCounts = [([u8; 6], u16); DEAUTH_SLOTS];
/// Size of `DeauthCounts` (the sniffer's counter table).
pub const DEAUTH_SLOTS: usize = 8;
/// Advertisements handled per loop turn.
const BLE_BATCH: usize = 32;
/// Handshake frames handled per loop turn (a real handshake is 2-4 frames; this is generous headroom).
const HANDSHAKE_BATCH: usize = 16;
/// Probe requests handled per loop turn.
const PROBE_BATCH: usize = 32;
/// Frames handled per loop turn, so a flood cannot delay the next command for long.
const FRAME_BATCH: usize = 32;

/// The adapter-side actions the app task needs, as plain function pointers so this file stays free of
/// adapter types.
pub struct Hooks {
    /// Fill the deauth counts gathered since the last call; returns how many entries are valid.
    pub drain_deauth: fn(&mut DeauthCounts) -> usize,
    /// Mirror the owner's BLE-monitor switch into the observer thread.
    pub set_ble_observer: fn(bool),
    /// Mirror the owner's handshake-capture switch into the sniffer callback.
    pub set_handshake_capture: fn(bool),
    /// Apply the owner's log verbosity.
    pub set_log_level: fn(LogLevel),
}

/// The single task that owns `App`. All state lives here, so no locks are needed.
/// Commands are served first; sniffed frames and the timers fill the gaps. The status heartbeat and the
/// deauth window follow the owner's settings (re-read each time they are scheduled).
pub fn run_app<R, W, S, T, P>(
    mut app: App<R, W, S, T, P>,
    events: Receiver<Event>,
    frames: Receiver<Observation>,
    ble: Receiver<AdClass>,
    handshake: Receiver<EapolFrame>,
    probes: Receiver<ProbeObservation>,
    hooks: Hooks,
) where
    R: RadioDriver,
    W: OtaWriter,
    S: System,
    T: TelemetrySink,
    P: SettingsStore,
{
    app.start(); // apply the saved settings (channel, filters, scan at boot)
    (hooks.set_log_level)(app.log_level());

    let ms = |n: u64| Duration::from_millis(n);
    let mut next_tick = Instant::now() + ms(u64::from(app.status_period_ms()));
    let mut next_deauth = Instant::now() + ms(u64::from(app.deauth_window_ms()));
    let mut log_level = app.log_level();
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
        for _ in 0..HANDSHAKE_BATCH {
            match handshake.try_recv() {
                Ok(f) => app.on_eapol_frame(f),
                Err(_) => break,
            }
        }
        for _ in 0..PROBE_BATCH {
            match probes.try_recv() {
                Ok(p) => app.on_probe(p),
                Err(_) => break,
            }
        }
        let now = Instant::now();
        if now >= next_deauth {
            let mut counts: DeauthCounts = [([0; 6], 0); DEAUTH_SLOTS];
            let n = (hooks.drain_deauth)(&mut counts);
            if n > 0 {
                app.on_deauth_window(&counts[..n]);
            }
            next_deauth = now + ms(u64::from(app.deauth_window_ms()));
        }
        if now >= next_tick {
            (hooks.set_ble_observer)(app.ble_monitor_enabled());
            (hooks.set_handshake_capture)(app.handshake_capture_enabled());
            if app.log_level() != log_level {
                log_level = app.log_level();
                (hooks.set_log_level)(log_level);
            }
            app.on_tick();
            next_tick = now + ms(u64::from(app.status_period_ms()));
        }
    }
}
