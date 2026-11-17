use beewave_protocol::{DeviceCommand, TelemetryMessage, AUTH_NONCE_LEN};

use super::keys::DeviceKeys;
use super::messages::{
    event, ota_error_text, radio_error_text, EVENT_OTA_ERROR, EVENT_RADIO_ERROR,
    EVENT_SETTINGS_ERROR, EVENT_UNAUTHORIZED,
};
use crate::features::auth::AuthGate;
use crate::features::ota::OtaSession;
use crate::features::radio::{Mode, RadioController};
use crate::features::scan::{Observation, ScanTable};
use crate::features::settings::SettingsService;
use crate::features::status;
use crate::ports::{OtaWriter, RadioDriver, Settings, SettingsStore, System, TelemetrySink};

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
    auth: AuthGate,
    radio: RadioController<R>,
    ota: OtaSession<W>,
    settings: SettingsService<P>,
    scans: ScanTable,
    sys: S,
    sink: T,
    dropped: u16,
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
            sys,
            sink,
            dropped: 0,
        }
    }

    /// Messages the sink refused since boot (reported in `Status.dropped`).
    pub fn dropped(&self) -> u16 {
        self.dropped
    }

    /// Apply the saved settings (channel, auto-scan). Call once before processing input.
    pub fn start(&mut self) {
        let s = self.settings.current();
        self.apply_channel(s);
        if s.auto_scan {
            self.radio
                .handle(&DeviceCommand::StartScan { band_mask: 1 });
        }
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
        }
        if matches!(cmd, DeviceCommand::RebootDevice) {
            self.sys.restart();
            return;
        }
        if let Some(result) = self.settings.handle(&cmd) {
            match result {
                Ok(s) => {
                    if matches!(cmd, DeviceCommand::SetSettings { .. }) {
                        self.apply_channel(s);
                    }
                    self.emit(TelemetryMessage::Settings {
                        channel: s.channel,
                        auto_scan: s.auto_scan,
                    });
                }
                Err(_) => self.emit(event(EVENT_SETTINGS_ERROR, "settings error")),
            }
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
        if let Some(m) = self.scans.observe(o) {
            self.emit(m);
        }
    }

    /// Driving port: periodic tick (status heartbeat).
    pub fn on_tick(&mut self) {
        let scanning = self.radio.mode() == Mode::Scanning;
        let m = status::sample(&self.sys, self.radio.channel(), self.dropped, scanning);
        self.emit(m);
    }

    /// Claim the device for `key`: only while unclaimed, and the key is persisted *before* the gate
    /// accepts it, so a storage failure leaves the device unclaimed rather than claimed-but-forgetful.
    fn provision(&mut self, key: [u8; 32]) {
        let ok = !self.auth.is_claimed() && key != [0u8; 32];
        let ok = ok && self.settings.save_key(&key).is_ok();
        let ok = ok && self.auth.claim(key);
        self.emit(TelemetryMessage::AuthResult { ok });
    }

    fn apply_channel(&mut self, s: Settings) {
        self.radio
            .handle(&DeviceCommand::ConfigureRadio { channel: s.channel });
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

    fn emit(&mut self, m: TelemetryMessage) {
        if !self.permitted(&m) {
            return; // withheld on purpose, not a drop
        }
        if !self.sink.publish(m) {
            self.dropped = self.dropped.saturating_add(1);
        }
    }
}
