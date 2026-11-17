//! `App`'s handling of settings, storage, diagnostics and device-info commands. Split from `service.rs`
//! to keep that file about routing; it relies on the fields being visible within `app`.
use beewave_protocol::{DeviceCommand, RejectReason, TelemetryMessage, SETTINGS_SCHEMA};
use heapless::String;

use super::messages::{event, EVENT_SETTINGS_ERROR};
use super::service::App;
use crate::features::radio::Mode;
use crate::features::settings::hop_order;
use crate::ports::{
    OtaWriter, RadioDriver, Settings, SettingsStore, System, TelemetrySink, AUTO_CHANNEL,
};

impl<R, W, S, T, P> App<R, W, S, T, P>
where
    R: RadioDriver,
    W: OtaWriter,
    S: System,
    T: TelemetrySink,
    P: SettingsStore,
{
    /// Make the stored settings the live ones.
    pub(super) fn apply_settings(&mut self) {
        let s = self.settings.current().clone();
        self.scans.configure(&s.scan);
        self.monitoring
            .apply_alert_settings(s.alerts.deauth_threshold);
        self.apply_channel(&s);
    }

    fn apply_channel(&mut self, s: &Settings) {
        let channel = s.radio.default_channel;
        self.radio
            .handle(&DeviceCommand::ConfigureRadio { channel });
        debug_assert!(channel == AUTO_CHANNEL || (1..=13).contains(&channel));
    }

    /// Hand the driver the hop settings; it uses them from the next scan start.
    pub(super) fn configure_hop(&mut self) {
        let r = &self.settings.current().radio;
        let order = hop_order(r);
        let dwell = r.hop_dwell_ms;
        self.radio.configure_hop(dwell, &order);
    }

    fn settings_all(&mut self) -> TelemetryMessage {
        let revision = self.settings.revision();
        let s = self.settings.current().clone();
        TelemetryMessage::SettingsAll {
            schema: SETTINGS_SCHEMA,
            revision,
            scan: s.scan,
            radio: s.radio,
            telemetry: s.telemetry,
            alerts: s.alerts,
        }
    }

    /// True if `cmd` was a settings/storage/system command (and has been answered).
    pub(super) fn handle_settings_command(&mut self, cmd: &DeviceCommand) -> bool {
        match cmd {
            DeviceCommand::GetSettings => {
                let s = self.settings.current();
                let reply = TelemetryMessage::Settings {
                    channel: s.radio.default_channel,
                    auto_scan: s.scan.scan_at_boot,
                };
                self.emit(reply);
            }
            DeviceCommand::SetSettings { channel, auto_scan } => {
                match self.settings.set_legacy(*channel, *auto_scan) {
                    Ok(_) => {
                        self.apply_settings();
                        let s = self.settings.current();
                        let reply = TelemetryMessage::Settings {
                            channel: s.radio.default_channel,
                            auto_scan: s.scan.scan_at_boot,
                        };
                        self.emit(reply);
                    }
                    Err(_) => self.emit(event(EVENT_SETTINGS_ERROR, "settings error")),
                }
            }
            DeviceCommand::GetSettingsAll => {
                let all = self.settings_all();
                self.emit(all);
            }
            DeviceCommand::SaveSettingsAll {
                scan,
                radio,
                telemetry,
                alerts,
            } => {
                let next = Settings {
                    scan: scan.clone(),
                    radio: radio.clone(),
                    telemetry: telemetry.clone(),
                    alerts: alerts.clone(),
                };
                let reply = match self.settings.save_all(next) {
                    Ok(revision) => {
                        self.apply_settings();
                        TelemetryMessage::SettingsSaved {
                            revision,
                            ok: true,
                            reason: None,
                        }
                    }
                    Err(reason) => TelemetryMessage::SettingsSaved {
                        revision: self.settings.revision(),
                        ok: false,
                        reason: Some(reason),
                    },
                };
                self.emit(reply);
            }
            DeviceCommand::FactoryReset => {
                if self.ota.in_progress() {
                    self.emit(TelemetryMessage::Rejected {
                        reason: RejectReason::Busy,
                    });
                } else if self.settings.factory_reset().is_ok() {
                    self.sys.restart();
                } else {
                    self.emit(TelemetryMessage::Rejected {
                        reason: RejectReason::Storage,
                    });
                }
            }
            DeviceCommand::RunSelfTest => {
                let reply = self.self_test();
                self.emit(reply);
            }
            DeviceCommand::GetStorageInfo => {
                let (nvs_used, nvs_free) = self.sys.nvs_usage();
                let p = self.sys.partition_info();
                let settings_revision = self.settings.revision();
                self.emit(TelemetryMessage::StorageInfo {
                    nvs_used,
                    nvs_free,
                    ota0_active: p.ota0_active,
                    ota1_active: p.ota1_active,
                    ota0_size: p.ota0_size,
                    ota1_size: p.ota1_size,
                    settings_revision,
                });
            }
            DeviceCommand::GetDeviceInfo => {
                let f = self.sys.build_facts();
                self.emit(TelemetryMessage::DeviceInfo {
                    version: text(f.version),
                    git_sha: text(f.git_sha),
                    built: text(f.built),
                    ota_key_fp: f.ota_key_fp,
                    request_conn_params: f.request_conn_params,
                    coex_debug: f.coex_debug,
                    bonding: f.bonding,
                });
            }
            _ => return false,
        }
        true
    }

    /// Runs the short diagnostic. While a scan is running nothing is touched (the sniffer owns the radio).
    fn self_test(&mut self) -> TelemetryMessage {
        if self.radio.mode() == Mode::Scanning {
            return TelemetryMessage::SelfTestResult {
                radio: false,
                nvs: false,
                ble: false,
                ota_slot: false,
                busy: true,
            };
        }
        TelemetryMessage::SelfTestResult {
            radio: self.radio.self_test(),
            nvs: self.sys.self_test_nvs(),
            ble: self.sys.self_test_ble(),
            ota_slot: self.sys.self_test_ota_slot(),
            busy: false,
        }
    }
}

/// `s` as a wire string of at most `N` bytes, cut at a character boundary.
fn text<const N: usize>(s: &str) -> String<N> {
    let mut out = String::new();
    for c in s.chars() {
        if out.push(c).is_err() {
            break;
        }
    }
    out
}
