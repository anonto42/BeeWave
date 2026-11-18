//! Composes the receive-only monitors (deauth, BLE, anomaly) and their configuration. Pure: the
//! firmware feeds it counts, advertisements and beacons and publishes what comes out.
use beewave_protocol::{AdClass, TelemetryMessage};
use heapless::Vec;

use crate::features::anomaly::AnomalyDetector;
use crate::features::ble_monitor::BleMonitor;
use crate::features::deauth_detect::{DeauthDetector, DEFAULT_THRESHOLD};
use crate::features::handshake::{extract_pmkid, EapolFrame, HandshakeTracker};
use crate::features::scan::Observation;

/// Allowed `ble_spike_factor` values.
pub const FACTOR_RANGE: core::ops::RangeInclusive<u8> = 2..=20;
const DEFAULT_FACTOR: u8 = 3;

/// What the owner can configure. Held in RAM only: a reboot restores the defaults (BLE monitor off).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorConfig {
    /// Deauth frames per second per BSSID that raise an alert; 0 = detector off.
    pub deauth_threshold: u8,
    pub ble_enabled: bool,
    pub ble_spike_factor: u8,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            deauth_threshold: DEFAULT_THRESHOLD,
            ble_enabled: false,
            ble_spike_factor: DEFAULT_FACTOR,
        }
    }
}

/// `SetMonitor` carried a value outside its range.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidMonitorConfig;

pub struct Monitoring {
    cfg: MonitorConfig,
    deauth: DeauthDetector,
    ble: BleMonitor,
    anomaly: AnomalyDetector,
    handshake: HandshakeTracker,
    /// Off by default, like the BLE monitor; kept outside `MonitorConfig` since it has its own
    /// command pair (`SetHandshakeCapture` / `GetHandshakeCapture`) rather than `SetMonitor`.
    handshake_enabled: bool,
    tick: u32,
}

impl Monitoring {
    pub fn new() -> Self {
        let cfg = MonitorConfig::default();
        let mut m = Self {
            cfg,
            deauth: DeauthDetector::new(),
            ble: BleMonitor::new(),
            anomaly: AnomalyDetector::new(),
            handshake: HandshakeTracker::new(),
            handshake_enabled: false,
            tick: 0,
        };
        m.apply(cfg);
        m
    }

    pub fn config(&self) -> MonitorConfig {
        self.cfg
    }

    pub fn report(&self) -> TelemetryMessage {
        TelemetryMessage::MonitorConfig {
            deauth_threshold: self.cfg.deauth_threshold,
            ble_enabled: self.cfg.ble_enabled,
            ble_spike_factor: self.cfg.ble_spike_factor,
        }
    }

    /// Validate and apply. On error nothing changes.
    pub fn set(&mut self, cfg: MonitorConfig) -> Result<(), InvalidMonitorConfig> {
        if !FACTOR_RANGE.contains(&cfg.ble_spike_factor) {
            return Err(InvalidMonitorConfig);
        }
        self.apply(cfg);
        Ok(())
    }

    fn apply(&mut self, cfg: MonitorConfig) {
        self.deauth.set_threshold(cfg.deauth_threshold);
        self.ble.set_factor(cfg.ble_spike_factor);
        if cfg.ble_enabled && !self.cfg.ble_enabled {
            self.ble.reset(); // a fresh baseline each time it is switched on
        }
        self.cfg = cfg;
    }

    /// A beacon the sniffer heard (before de-duplication).
    pub fn on_observation(&mut self, o: &Observation) -> Option<TelemetryMessage> {
        self.anomaly
            .observe(o, self.tick)
            .map(|a| TelemetryMessage::AnomalyAlert {
                kind: a.kind,
                bssid: a.bssid,
                detail: a.detail,
            })
    }

    /// The owner's persisted deauth threshold became the active one (a later `SetMonitor` may override it in RAM).
    pub fn apply_alert_settings(&mut self, deauth_threshold: u8) {
        self.deauth.set_threshold(deauth_threshold);
        self.cfg.deauth_threshold = deauth_threshold;
    }

    /// Deauth frame counts per BSSID gathered over `window_ms`. Returns the alerts to publish.
    pub fn on_deauth_window(
        &self,
        counts: &[([u8; 6], u16)],
        window_ms: u16,
    ) -> Vec<TelemetryMessage, 4> {
        let mut out = Vec::new();
        for hit in self.deauth.evaluate_window(counts, window_ms) {
            let _ = out.push(TelemetryMessage::DeauthAlert {
                bssid: hit.bssid,
                count: hit.count,
            });
        }
        out
    }

    pub fn handshake_capture_enabled(&self) -> bool {
        self.handshake_enabled
    }

    /// Turn the passive handshake listener on or off; a fresh start forgets every pair in progress.
    pub fn set_handshake_capture(&mut self, enabled: bool) {
        if enabled && !self.handshake_enabled {
            self.handshake.reset();
        }
        self.handshake_enabled = enabled;
    }

    /// A classified EAPOL-Key frame. Ignored while the listener is off. Returns the frame telemetry
    /// (always, when on) and the completion telemetry (only the moment a pair becomes complete).
    pub fn on_eapol_frame(&mut self, f: EapolFrame) -> Vec<TelemetryMessage, 3> {
        let mut out = Vec::new();
        if !self.handshake_enabled {
            return out;
        }
        // Message 1 (AP → STA, ACK set, MIC clear) is the only message that carries a PMKID KDE.
        // Extract it here, before `f.eapol` moves into HandshakeFrame telemetry.
        if f.message == 1 {
            if let Some(pmkid) = extract_pmkid(&f.eapol) {
                if let Some(capture) = self.handshake.observe_pmkid(f.bssid, f.client, pmkid) {
                    let _ = out.push(TelemetryMessage::PmkidFound {
                        bssid: capture.bssid,
                        client: capture.client,
                        pmkid: capture.pmkid,
                    });
                }
            }
        }
        let _ = out.push(TelemetryMessage::HandshakeFrame {
            bssid: f.bssid,
            client: f.client,
            message: f.message,
            replay_counter: f.replay_counter,
            truncated: f.truncated,
            eapol: f.eapol.clone(),
        });
        if let Some(done) = self.handshake.observe(&f) {
            let _ = out.push(TelemetryMessage::HandshakeComplete {
                bssid: done.bssid,
                client: done.client,
            });
        }
        out
    }

    /// A received advertisement, already classified. Ignored while the monitor is off.
    pub fn on_ble_adv(&mut self, class: AdClass) {
        if self.cfg.ble_enabled {
            self.ble.observe(class);
        }
    }

    /// 1 Hz: advance time; returns the BLE stats and spike to publish, if any.
    pub fn on_tick(&mut self) -> Vec<TelemetryMessage, 2> {
        self.tick = self.tick.wrapping_add(1);
        let mut out = Vec::new();
        if !self.cfg.ble_enabled {
            return out;
        }
        let t = self.ble.tick();
        if let Some([fast_pair, ibeacon, proximity, generic]) = t.stats {
            let _ = out.push(TelemetryMessage::BleStats {
                fast_pair,
                ibeacon,
                proximity,
                generic,
            });
        }
        if let Some(s) = t.spike {
            let _ = out.push(TelemetryMessage::BleSpike {
                class: s.class,
                count: s.count,
                baseline: s.baseline,
            });
        }
        out
    }
}

impl Default for Monitoring {
    fn default() -> Self {
        Self::new()
    }
}
