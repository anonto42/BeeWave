extern crate std;
use std::{cell::RefCell, rc::Rc, vec::Vec};

use beewave_protocol::{DeviceCommand, TelemetryMessage};

use super::*;
use crate::features::auth::proof;
use crate::features::scan::Observation;
use crate::ports::{
    OtaError, OtaWriter, RadioDriver, RadioError, Settings, SettingsError, SettingsStore, System,
    TelemetrySink,
};

#[derive(Default)]
struct FakeRadio(u8);
impl RadioDriver for FakeRadio {
    fn channel(&self) -> u8 {
        self.0
    }
    fn set_channel(&mut self, c: u8) -> Result<(), RadioError> {
        self.0 = c;
        Ok(())
    }
    fn start_sniffing(&mut self) -> Result<(), RadioError> {
        Ok(())
    }
    fn stop_sniffing(&mut self) -> Result<(), RadioError> {
        Ok(())
    }
}

#[derive(Default)]
struct FakeStore {
    settings: Option<Settings>,
    /// Shared so a test can see what the device persisted after the store moved into the `App`.
    key: Rc<RefCell<Option<[u8; 32]>>>,
    key_save_fails: bool,
}
impl SettingsStore for FakeStore {
    fn load(&mut self) -> Settings {
        self.settings.unwrap_or_default()
    }
    fn save(&mut self, s: &Settings) -> Result<(), SettingsError> {
        self.settings = Some(*s);
        Ok(())
    }
    fn load_key(&mut self) -> Option<[u8; 32]> {
        *self.key.borrow()
    }
    fn save_key(&mut self, key: &[u8; 32]) -> Result<(), SettingsError> {
        if self.key_save_fails {
            return Err(SettingsError::Storage);
        }
        *self.key.borrow_mut() = Some(*key);
        Ok(())
    }
}

struct FakeOta;
impl OtaWriter for FakeOta {
    fn begin(&mut self, _: u32) -> Result<(), OtaError> {
        Ok(())
    }
    fn write(&mut self, _: &[u8]) -> Result<(), OtaError> {
        Ok(())
    }
    fn finish(&mut self, _: &[u8; 32]) -> Result<(), OtaError> {
        Ok(())
    }
    fn abort(&mut self) {}
}

/// `Rc<RefCell<..>>` = shared, mutable-through-a-check; lets the test see what the fake saw.
struct FakeSystem(Rc<RefCell<bool>>);
impl System for FakeSystem {
    fn free_heap(&self) -> u32 {
        100
    }
    fn cpu_load(&self) -> u8 {
        1
    }
    fn restart(&mut self) {
        *self.0.borrow_mut() = true;
    }
    fn fill_random(&mut self, buf: &mut [u8]) {
        buf.fill(0x42);
    }
}

/// A sink that records messages, or refuses them all when `full` is true.
#[derive(Clone, Default)]
struct FakeSink {
    got: Rc<RefCell<Vec<TelemetryMessage>>>,
    full: bool,
}
impl TelemetrySink for FakeSink {
    fn publish(&mut self, m: TelemetryMessage) -> bool {
        if self.full {
            return false;
        }
        self.got.borrow_mut().push(m);
        true
    }
}

const KEY: [u8; 32] = [7; 32];

type TestApp = App<FakeRadio, FakeOta, FakeSystem, FakeSink, FakeStore>;

fn app_with(sink: FakeSink, rebooted: Rc<RefCell<bool>>, key: Option<[u8; 32]>) -> TestApp {
    app_with_store(sink, rebooted, key, FakeStore::default())
}

fn app_with_store(
    sink: FakeSink,
    rebooted: Rc<RefCell<bool>>,
    key: Option<[u8; 32]>,
    store: FakeStore,
) -> TestApp {
    App::new(
        FakeRadio::default(),
        FakeOta,
        FakeSystem(rebooted),
        sink,
        store,
        DeviceKeys {
            auth: key,
            ota_signing: None,
        },
    )
}

/// Run the handshake the way a client does, then forget what it produced.
fn authenticate(app: &mut TestApp, sink: &FakeSink) {
    app.on_command(DeviceCommand::AuthBegin);
    let mac = proof(&KEY, &[0x42; 16]).unwrap();
    app.on_command(DeviceCommand::AuthProve { mac });
    sink.got.borrow_mut().clear();
}

#[test]
fn routes_commands_and_observations() {
    let sink = FakeSink::default();
    let rebooted = Rc::new(RefCell::new(false));
    let mut app = app_with(sink.clone(), rebooted.clone(), Some(KEY));
    authenticate(&mut app, &sink);

    app.on_command(DeviceCommand::ConfigureRadio { channel: 99 });
    app.on_command(DeviceCommand::ConfigureRadio { channel: 6 });
    app.on_observation(Observation {
        bssid: [1; 6],
        channel: 6,
        rssi: -40,
        ssid: "a".try_into().unwrap(),
    });
    app.on_tick();

    let out = sink.got.borrow();
    assert!(matches!(out[0], TelemetryMessage::Event { code: 1, .. }));
    assert!(matches!(out[1], TelemetryMessage::ScanResult { .. }));
    assert!(matches!(
        out[2],
        TelemetryMessage::Status {
            active_channel: 6,
            ..
        }
    ));

    app.on_command(DeviceCommand::RebootDevice);
    assert!(*rebooted.borrow());
}

#[test]
fn status_reports_whether_a_scan_is_running() {
    let sink = FakeSink::default();
    let mut app = app_with(sink.clone(), Rc::default(), Some(KEY));
    authenticate(&mut app, &sink);
    let scanning = |sink: &FakeSink| match sink.got.borrow().last() {
        Some(TelemetryMessage::Status { scanning, .. }) => *scanning,
        other => panic!("expected a status, got {other:?}"),
    };

    app.on_tick();
    assert!(!scanning(&sink));
    app.on_command(DeviceCommand::StartScan { band_mask: 1 });
    app.on_tick();
    assert!(scanning(&sink));
    app.on_command(DeviceCommand::StopOperation);
    app.on_tick();
    assert!(!scanning(&sink));
}

#[test]
fn counts_drops_when_sink_full() {
    let sink = FakeSink {
        full: true,
        ..Default::default()
    };
    let mut app = app_with(sink, Default::default(), Some(KEY));
    // Authenticated, but the sink refuses everything, so only drops are counted.
    app.on_command(DeviceCommand::AuthBegin);
    app.on_command(DeviceCommand::AuthProve {
        mac: proof(&KEY, &[0x42; 16]).unwrap(),
    });
    let before = app.dropped();
    app.on_tick();
    app.on_tick();
    assert_eq!(app.dropped(), before + 2);
}

#[test]
fn start_applies_saved_settings_and_set_persists() {
    let sink = FakeSink::default();
    let store = FakeStore {
        settings: Some(Settings {
            channel: 9,
            auto_scan: true,
        }),
        ..Default::default()
    };
    let mut app = App::new(
        FakeRadio::default(),
        FakeOta,
        FakeSystem(Default::default()),
        sink.clone(),
        store,
        DeviceKeys {
            auth: Some(KEY),
            ota_signing: None,
        },
    );
    authenticate(&mut app, &sink);
    app.start();
    app.on_tick();
    assert!(matches!(
        sink.got.borrow()[0],
        TelemetryMessage::Status {
            active_channel: 9,
            ..
        }
    ));

    app.on_command(DeviceCommand::SetSettings {
        channel: 4,
        auto_scan: false,
    });
    app.on_command(DeviceCommand::GetSettings);
    let out = sink.got.borrow();
    assert!(matches!(
        out[1],
        TelemetryMessage::Settings {
            channel: 4,
            auto_scan: false
        }
    ));
    assert!(matches!(
        out[2],
        TelemetryMessage::Settings {
            channel: 4,
            auto_scan: false
        }
    ));
}

#[test]
fn rejects_commands_before_authentication() {
    let sink = FakeSink::default();
    let rebooted = Rc::new(RefCell::new(false));
    let mut app = app_with(sink.clone(), rebooted.clone(), Some(KEY));

    app.on_command(DeviceCommand::RebootDevice);
    app.on_command(DeviceCommand::ConfigureRadio { channel: 6 });

    assert!(!*rebooted.borrow(), "reboot must not run unauthenticated");
    let out = sink.got.borrow();
    assert_eq!(out.len(), 2);
    assert!(out
        .iter()
        .all(|m| matches!(m, TelemetryMessage::Event { code: 4, .. })));
}

#[test]
fn withholds_telemetry_until_authenticated() {
    let sink = FakeSink::default();
    let mut app = app_with(sink.clone(), Default::default(), Some(KEY));

    app.on_tick();
    app.on_observation(Observation {
        bssid: [1; 6],
        channel: 6,
        rssi: -40,
        ssid: "a".try_into().unwrap(),
    });
    assert!(sink.got.borrow().is_empty());
    assert_eq!(app.dropped(), 0, "withheld is not dropped");

    authenticate(&mut app, &sink);
    app.on_tick();
    assert!(matches!(
        sink.got.borrow()[0],
        TelemetryMessage::Status { .. }
    ));
}

#[test]
fn handshake_returns_challenge_then_result() {
    let sink = FakeSink::default();
    let mut app = app_with(sink.clone(), Default::default(), Some(KEY));
    app.on_command(DeviceCommand::AuthBegin);
    app.on_command(DeviceCommand::AuthProve {
        mac: proof(&KEY, &[0x42; 16]).unwrap(),
    });
    let out = sink.got.borrow();
    assert_eq!(
        out[0],
        TelemetryMessage::AuthChallenge { nonce: [0x42; 16] }
    );
    assert_eq!(out[1], TelemetryMessage::AuthResult { ok: true });
}

#[test]
fn link_reset_requires_a_new_handshake() {
    let sink = FakeSink::default();
    let rebooted = Rc::new(RefCell::new(false));
    let mut app = app_with(sink.clone(), rebooted.clone(), Some(KEY));
    authenticate(&mut app, &sink);

    app.on_link_reset();
    app.on_command(DeviceCommand::RebootDevice);

    assert!(!*rebooted.borrow());
    assert!(matches!(
        sink.got.borrow()[0],
        TelemetryMessage::Event { code: 4, .. }
    ));
}

#[test]
fn unclaimed_device_rejects_proofs_and_commands() {
    let sink = FakeSink::default();
    let rebooted = Rc::new(RefCell::new(false));
    let mut app = app_with(sink.clone(), rebooted.clone(), None);

    app.on_command(DeviceCommand::AuthBegin);
    app.on_command(DeviceCommand::AuthProve {
        mac: proof(&KEY, &[0x42; 16]).unwrap(),
    });
    app.on_command(DeviceCommand::RebootDevice);

    assert!(!*rebooted.borrow());
    assert!(sink
        .got
        .borrow()
        .iter()
        .any(|m| *m == TelemetryMessage::AuthResult { ok: false }));
}

fn results(sink: &FakeSink) -> Vec<TelemetryMessage> {
    sink.got.borrow().clone()
}

#[test]
fn unclaimed_device_announces_itself() {
    let sink = FakeSink::default();
    let mut app = app_with(sink.clone(), Rc::default(), None);
    app.on_command(DeviceCommand::AuthBegin);
    assert_eq!(results(&sink), [TelemetryMessage::Unclaimed]);
}

#[test]
fn first_client_claims_the_device_and_the_key_is_stored() {
    let sink = FakeSink::default();
    let store = FakeStore::default();
    let stored = store.key.clone();
    let mut app = app_with_store(sink.clone(), Rc::default(), None, store);
    app.on_command(DeviceCommand::Provision { key: KEY });

    assert_eq!(results(&sink), [TelemetryMessage::AuthResult { ok: true }]);
    assert_eq!(*stored.borrow(), Some(KEY));
    // the claiming session is authenticated: status flows
    sink.got.borrow_mut().clear();
    app.on_tick();
    assert!(matches!(
        sink.got.borrow()[0],
        TelemetryMessage::Status { .. }
    ));

    // a second client must prove the new key; announcing itself as unclaimed no longer happens
    app.on_link_reset();
    sink.got.borrow_mut().clear();
    app.on_command(DeviceCommand::AuthBegin);
    assert!(matches!(
        sink.got.borrow()[0],
        TelemetryMessage::AuthChallenge { .. }
    ));
}

#[test]
fn a_claimed_device_refuses_to_be_reclaimed() {
    let sink = FakeSink::default();
    let store = FakeStore::default();
    let stored = store.key.clone();
    let mut app = app_with_store(sink.clone(), Rc::default(), Some(KEY), store);
    app.on_command(DeviceCommand::Provision { key: [7; 32] });

    assert_eq!(results(&sink), [TelemetryMessage::AuthResult { ok: false }]);
    assert_eq!(*stored.borrow(), None);
    assert!(!app_is_authenticated(&mut app, &sink));
}

#[test]
fn storage_failure_leaves_the_device_unclaimed() {
    let sink = FakeSink::default();
    let store = FakeStore {
        key_save_fails: true,
        ..Default::default()
    };
    let mut app = app_with_store(sink.clone(), Rc::default(), None, store);
    app.on_command(DeviceCommand::Provision { key: KEY });
    assert_eq!(results(&sink), [TelemetryMessage::AuthResult { ok: false }]);

    sink.got.borrow_mut().clear();
    app.on_command(DeviceCommand::AuthBegin);
    assert_eq!(results(&sink), [TelemetryMessage::Unclaimed]);
}

#[test]
fn the_all_zero_key_is_refused() {
    let sink = FakeSink::default();
    let mut app = app_with(sink.clone(), Rc::default(), None);
    app.on_command(DeviceCommand::Provision { key: [0; 32] });
    assert_eq!(results(&sink), [TelemetryMessage::AuthResult { ok: false }]);
}

#[test]
fn a_stored_key_survives_a_reboot() {
    let sink = FakeSink::default();
    let store = FakeStore::default();
    *store.key.borrow_mut() = Some(KEY);
    let mut app = app_with_store(sink.clone(), Rc::default(), None, store);
    app.on_command(DeviceCommand::AuthBegin);
    assert!(matches!(
        sink.got.borrow()[0],
        TelemetryMessage::AuthChallenge { .. }
    ));
}

#[test]
fn a_build_time_key_wins_over_the_stored_one() {
    let sink = FakeSink::default();
    let store = FakeStore::default();
    *store.key.borrow_mut() = Some([9; 32]);
    let mut app = app_with_store(sink.clone(), Rc::default(), Some(KEY), store);
    authenticate(&mut app, &sink); // proves with KEY
    app.on_tick();
    assert!(matches!(
        sink.got.borrow()[0],
        TelemetryMessage::Status { .. }
    ));
}

/// True if the session can run a command (authenticated).
fn app_is_authenticated(app: &mut TestApp, sink: &FakeSink) -> bool {
    sink.got.borrow_mut().clear();
    app.on_tick();
    !sink.got.borrow().is_empty()
}

// ---- passive monitoring ------------------------------------------------------------------------

use beewave_protocol::{AdClass, AnomalyKind};

fn authed_app() -> (TestApp, FakeSink) {
    let sink = FakeSink::default();
    let mut app = app_with(sink.clone(), Rc::default(), Some(KEY));
    authenticate(&mut app, &sink);
    (app, sink)
}

fn set_monitor(app: &mut TestApp, deauth: u8, ble: bool, factor: u8) {
    app.on_command(DeviceCommand::SetMonitor {
        deauth_threshold: deauth,
        ble_enabled: ble,
        ble_spike_factor: factor,
    });
}

#[test]
fn monitor_config_defaults_and_round_trips() {
    let (mut app, sink) = authed_app();
    app.on_command(DeviceCommand::GetMonitor);
    assert_eq!(
        results(&sink),
        [TelemetryMessage::MonitorConfig {
            deauth_threshold: 10,
            ble_enabled: false,
            ble_spike_factor: 3
        }]
    );
    assert!(!app.ble_monitor_enabled());

    sink.got.borrow_mut().clear();
    set_monitor(&mut app, 25, true, 5);
    assert_eq!(
        results(&sink),
        [TelemetryMessage::MonitorConfig {
            deauth_threshold: 25,
            ble_enabled: true,
            ble_spike_factor: 5
        }]
    );
    assert!(app.ble_monitor_enabled());
}

#[test]
fn an_out_of_range_factor_is_refused_and_changes_nothing() {
    let (mut app, sink) = authed_app();
    set_monitor(&mut app, 25, true, 99);
    assert!(matches!(
        results(&sink)[0],
        TelemetryMessage::Event { code: 5, .. }
    ));
    assert!(!app.ble_monitor_enabled(), "the whole request is rejected");
}

#[test]
fn monitor_commands_need_an_authenticated_session() {
    let sink = FakeSink::default();
    let mut app = app_with(sink.clone(), Rc::default(), Some(KEY));
    set_monitor(&mut app, 1, true, 3);
    assert!(matches!(
        results(&sink)[0],
        TelemetryMessage::Event { code: 4, .. }
    ));
    assert!(!app.ble_monitor_enabled());
}

#[test]
fn deauth_counts_at_the_threshold_raise_an_alert() {
    let (mut app, sink) = authed_app();
    app.on_deauth_second(&[([1; 6], 9), ([2; 6], 12)]);
    assert_eq!(
        results(&sink),
        [TelemetryMessage::DeauthAlert {
            bssid: [2; 6],
            count: 12
        }]
    );
}

#[test]
fn deauth_alerts_follow_the_configured_threshold() {
    let (mut app, sink) = authed_app();
    set_monitor(&mut app, 0, false, 3);
    sink.got.borrow_mut().clear();
    app.on_deauth_second(&[([2; 6], 5000)]);
    assert!(results(&sink).is_empty(), "0 = off");
    set_monitor(&mut app, 3, false, 3);
    sink.got.borrow_mut().clear();
    app.on_deauth_second(&[([2; 6], 3)]);
    assert_eq!(results(&sink).len(), 1);
}

#[test]
fn deauth_alerts_are_withheld_until_authenticated() {
    let sink = FakeSink::default();
    let mut app = app_with(sink.clone(), Rc::default(), Some(KEY));
    app.on_deauth_second(&[([2; 6], 500)]);
    assert!(results(&sink).is_empty());
}

#[test]
fn ble_advertisements_are_ignored_while_the_monitor_is_off() {
    let (mut app, sink) = authed_app();
    for _ in 0..30 {
        app.on_ble_adv(AdClass::Generic);
        app.on_tick();
    }
    assert!(!results(&sink)
        .iter()
        .any(|m| matches!(m, TelemetryMessage::BleStats { .. })));
}

#[test]
fn ble_stats_are_published_every_ten_seconds_while_on() {
    let (mut app, sink) = authed_app();
    set_monitor(&mut app, 10, true, 3);
    sink.got.borrow_mut().clear();
    for _ in 0..10 {
        app.on_ble_adv(AdClass::IBeacon);
        app.on_tick();
    }
    let stats: Vec<_> = results(&sink)
        .into_iter()
        .filter(|m| matches!(m, TelemetryMessage::BleStats { .. }))
        .collect();
    assert_eq!(
        stats,
        [TelemetryMessage::BleStats {
            fast_pair: 0,
            ibeacon: 10,
            proximity: 0,
            generic: 0
        }]
    );
}

#[test]
fn switching_the_ble_monitor_on_starts_from_a_clean_slate() {
    let (mut app, sink) = authed_app();
    set_monitor(&mut app, 10, true, 3);
    for _ in 0..5 {
        app.on_ble_adv(AdClass::Generic);
    }
    set_monitor(&mut app, 10, false, 3);
    set_monitor(&mut app, 10, true, 3);
    sink.got.borrow_mut().clear();
    for _ in 0..10 {
        app.on_tick();
    }
    assert!(results(&sink).iter().any(|m| *m
        == TelemetryMessage::BleStats {
            fast_pair: 0,
            ibeacon: 0,
            proximity: 0,
            generic: 0
        }));
}

#[test]
fn a_known_ssid_on_a_new_bssid_publishes_an_anomaly_after_the_learning_period() {
    let (mut app, sink) = authed_app();
    let beacon = |id: u8| Observation {
        bssid: [id; 6],
        channel: 6,
        rssi: -50,
        ssid: "home".try_into().unwrap(),
    };
    app.on_observation(beacon(1));
    for _ in 0..40 {
        app.on_tick();
    }
    sink.got.borrow_mut().clear();
    app.on_observation(beacon(2));
    assert!(results(&sink).iter().any(|m| matches!(
        m,
        TelemetryMessage::AnomalyAlert {
            kind: AnomalyKind::NewBssidForSsid,
            bssid,
            ..
        } if *bssid == [2; 6]
    )));
}
