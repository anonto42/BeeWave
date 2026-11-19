use beewave_protocol::{AdClass, DeviceCommand, LogLevel, TelemetryMessage, AUTH_NONCE_LEN};

use super::keys::DeviceKeys;
use super::messages::{
    event, ota_error_text, radio_error_text, EVENT_MONITOR_ERROR, EVENT_OTA_ERROR,
    EVENT_RADIO_ERROR, EVENT_UNAUTHORIZED,
};
use super::monitoring::{MonitorConfig, Monitoring};
use crate::features::auth::AuthGate;
use crate::features::ota::OtaSession;
use crate::features::radio::{Mode, RadioController};
use crate::features::scan::{Observation, ScanTable};
use crate::features::settings::SettingsService;
use crate::features::status;
use crate::ports::{OtaWriter, RadioDriver, SettingsStore, System, TelemetrySink};

/// Composes the feature slices. The five generic parameters are the five driven ports;
/// the firmware plugs in real adapters, tests plug in fakes.
pub struct App<R, W, S, T, P>
where
    R: RadioDriver,
    W: OtaWriter,
    S: System,
    T: TelemetrySink,
    P: SettingsStore,
{
    pub(super) auth: AuthGate,
    pub(super) radio: RadioController<R>,
    pub(super) ota: OtaSession<W>,
    pub(super) settings: SettingsService<P>,
    pub(super) scans: ScanTable,
    pub(super) monitoring: Monitoring,
    pub(super) sys: S,
    pub(super) sink: T,
    dropped: u16,
    /// Running time in milliseconds, advanced by the status period on every tick.
    elapsed_ms: u32,
}

impl<R, W, S, T, P> App<R, W, S, T, P>
where
    R: RadioDriver,
    W: OtaWriter,
    S: System,
    T: TelemetrySink,
    P: SettingsStore,
{
    /// `keys` are the build-time secrets. Without a build-time or stored auth key the device is
    /// unclaimed: the first client to `Provision` one owns it (see `AuthGate::claim`).
    pub fn new(radio: R, ota: W, sys: S, sink: T, settings_store: P, keys: DeviceKeys) -> Self {
        let mut settings = SettingsService::new(settings_store);
        // A build-time key wins; otherwise the key a client provisioned earlier; otherwise unclaimed.
        let auth_key = keys.auth.or_else(|| settings.load_key());
        Self {
            auth: AuthGate::new(auth_key),
            radio: RadioController::new(radio),
            ota: OtaSession::new(ota, keys.ota_signing),
            settings,
            scans: ScanTable::new(),
            monitoring: Monitoring::new(),
            sys,
            sink,
            dropped: 0,
            elapsed_ms: 0,
        }
    }

    /// Messages the sink refused since boot (reported in `Status.dropped`).
    pub fn dropped(&self) -> u16 {
        self.dropped
    }

    /// Apply the saved settings (scan policy, alert thresholds, channel, scan at boot). Call once before
    /// processing input.
    pub fn start(&mut self) {
        self.apply_settings();
        if self.settings.current().scan.scan_at_boot {
            self.configure_hop();
            self.radio
                .handle(&DeviceCommand::StartScan { band_mask: 1 });
        }
    }

    /// Milliseconds between status heartbeats (the firmware schedules `on_tick` with it).
    pub fn status_period_ms(&self) -> u32 {
        self.settings.current().telemetry.status_period_ms
    }

    /// Window the deauth counters are drained over (the firmware schedules `on_deauth_window` with it).
    pub fn deauth_window_ms(&self) -> u16 {
        self.settings.current().alerts.deauth_window_ms
    }

    /// Verbosity the device's own logging should use.
    pub fn log_level(&self) -> LogLevel {
        self.settings.current().telemetry.log_level
    }

    /// Driving port: a command arrived from the client.
    pub fn on_command(&mut self, cmd: DeviceCommand) {
        match cmd {
            DeviceCommand::AuthBegin => {
                if !self.auth.is_claimed() {
                    self.emit(TelemetryMessage::Unclaimed);
                    return;
                }
                let mut nonce = [0u8; AUTH_NONCE_LEN];
                self.sys.fill_random(&mut nonce);
                let challenge = self.auth.begin(nonce);
                self.emit(challenge);
                return;
            }
            DeviceCommand::AuthProve { mac } => {
                let result = self.auth.prove(&mac);
                self.emit(result);
                return;
            }
            DeviceCommand::Provision { key } => {
                self.provision(key);
                return;
            }
            _ => {}
        }
        if !self.auth.is_authenticated() {
            self.emit(event(EVENT_UNAUTHORIZED, "not authenticated"));
            return;
        }
        if matches!(cmd, DeviceCommand::StartScan { .. }) {
            self.scans.clear();
            self.configure_hop(); // hopping changes apply when a scan starts, never mid-hop
        }
        if matches!(cmd, DeviceCommand::RebootDevice) {
            self.sys.restart();
            return;
        }
        match cmd {
            DeviceCommand::GetMonitor => {
                let report = self.monitoring.report();
                self.emit(report);
                return;
            }
            DeviceCommand::SetMonitor {
                deauth_threshold,
                ble_enabled,
                ble_spike_factor,
            } => {
                let cfg = MonitorConfig {
                    deauth_threshold,
                    ble_enabled,
                    ble_spike_factor,
                };
                match self.monitoring.set(cfg) {
                    Ok(()) => {
                        let report = self.monitoring.report();
                        self.emit(report);
                    }
                    Err(_) => self.emit(event(EVENT_MONITOR_ERROR, "monitor config out of range")),
                }
                return;
            }
            DeviceCommand::GetHandshakeCapture => {
                let enabled = self.monitoring.handshake_capture_enabled();
                self.emit(TelemetryMessage::HandshakeCaptureState { enabled });
                return;
            }
            DeviceCommand::SetHandshakeCapture { enabled } => {
                self.monitoring.set_handshake_capture(enabled);
                self.emit(TelemetryMessage::HandshakeCaptureState { enabled });
                return;
            }
            _ => {}
        }
        if self.handle_settings_command(&cmd) {
            return;
        }
        if let Some(result) = self.radio.handle(&cmd) {
            if let Err(e) = result {
                self.emit(event(EVENT_RADIO_ERROR, radio_error_text(e)));
            }
            return;
        }
        if let Some(result) = self.ota.handle(&cmd) {
            match result {
                Ok(Some(ack)) => self.emit(ack),
                Ok(None) => {}
                Err(e) => self.emit(event(EVENT_OTA_ERROR, ota_error_text(e))),
            }
        }
    }

    /// Driving port: a client connected or disconnected. Ends the session and any update.
    pub fn on_link_reset(&mut self) {
        self.auth.reset();
        self.ota.abort();
    }

    /// Driving port: the sniffer saw a frame.
    pub fn on_observation(&mut self, o: Observation) {
        if let Some(alert) = self.monitoring.on_observation(&o) {
            self.emit(alert);
        }
        if let Some(wps) = &o.wps {
            if let Some(m) = self.monitoring.on_wps(wps) {
                self.emit(m);
            }
        }
        if let Some(sec) = &o.security {
            if let Some(m) = self.monitoring.on_security(sec) {
                self.emit(m);
            }
        }
        if let Some(m) = self.scans.observe(o, self.elapsed_ms) {
            self.emit(m);
        }
    }

    /// Driving port: a parsed 802.11 Probe Request frame heard by the sniffer.
    pub fn on_probe(&mut self, p: crate::features::probe::ProbeObservation) {
        if let Some(m) = self.monitoring.on_probe(&p) {
            self.emit(m);
        }
    }

    /// Driving port: a parsed 802.11 Data frame association heard by the sniffer.
    pub fn on_assoc(&mut self, a: crate::features::assoc::AssocObservation) {
        if let Some(m) = self.monitoring.on_assoc(&a) {
            self.emit(m);
        }
    }

    /// Driving port: deauthentication frame counts per BSSID gathered over `deauth_window_ms`, drained from
    /// the sniffer's counters. Publishes an alert for each BSSID whose rate reaches the threshold.
    pub fn on_deauth_window(&mut self, counts: &[([u8; 6], u16)]) {
        let window = self.deauth_window_ms();
        for alert in self.monitoring.on_deauth_window(counts, window) {
            self.emit(alert);
        }
    }

    /// Driving port: disassociation frame counts per BSSID gathered over `deauth_window_ms`, drained from
    /// the sniffer's counters. Publishes an alert for each BSSID whose rate reaches the threshold.
    pub fn on_disassoc_window(&mut self, counts: &[([u8; 6], u16)]) {
        let window = self.deauth_window_ms();
        for alert in self.monitoring.on_disassoc_window(counts, window) {
            self.emit(alert);
        }
    }

    /// Driving port: a passively received BLE advertisement, already classified.
    pub fn on_ble_adv(&mut self, class: AdClass) {
        self.monitoring.on_ble_adv(class);
    }

    /// Driving port: a classified EAPOL-Key frame from a handshake happening on its own (nothing was
    /// sent to cause it). Ignored while the listener is off.
    pub fn on_eapol_frame(&mut self, f: crate::features::handshake::EapolFrame) {
        for m in self.monitoring.on_eapol_frame(f) {
            self.emit(m);
        }
    }

    /// Whether the passive handshake listener should be running (the firmware mirrors this; the radio
    /// stays receive-only either way).
    pub fn handshake_capture_enabled(&self) -> bool {
        self.monitoring.handshake_capture_enabled()
    }

    /// Whether the passive BLE observer should be running (the firmware mirrors this into it).
    pub fn ble_monitor_enabled(&self) -> bool {
        self.monitoring.config().ble_enabled
    }

    /// Driving port: periodic tick (status heartbeat, BLE monitor statistics).
    pub fn on_tick(&mut self) {
        self.elapsed_ms = self.elapsed_ms.wrapping_add(self.status_period_ms());
        let scanning = self.radio.mode() == Mode::Scanning;
        let m = status::sample(&self.sys, self.radio.channel(), self.dropped, scanning);
        self.emit(m);
        for m in self.monitoring.on_tick() {
            self.emit(m);
        }
    }

    /// Claim the device for `key`: only while unclaimed, and the key is persisted *before* the gate
    /// accepts it, so a storage failure leaves the device unclaimed rather than claimed-but-forgetful.
    fn provision(&mut self, key: [u8; 32]) {
        let ok = !self.auth.is_claimed() && key != [0u8; 32];
        let ok = ok && self.settings.save_key(&key).is_ok();
        let ok = ok && self.auth.claim(key);
        self.emit(TelemetryMessage::AuthResult { ok });
    }

    /// Until authenticated, only the handshake and the unauthorized event may leave the device.
    fn permitted(&self, m: &TelemetryMessage) -> bool {
        self.auth.is_authenticated()
            || matches!(
                m,
                TelemetryMessage::AuthChallenge { .. }
                    | TelemetryMessage::AuthResult { .. }
                    | TelemetryMessage::Unclaimed
                    | TelemetryMessage::Event {
                        code: EVENT_UNAUTHORIZED,
                        ..
                    }
            )
    }

    pub(super) fn emit(&mut self, m: TelemetryMessage) {
        if !self.permitted(&m) {
            return; // withheld on purpose, not a drop
        }
        if !self.sink.publish(m) {
            self.dropped = self.dropped.saturating_add(1);
        }
    }
}
