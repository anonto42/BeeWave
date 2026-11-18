extern crate std;
use std::{cell::RefCell, rc::Rc, vec::Vec};

use beewave_protocol::{DeviceCommand, TelemetryMessage};

use super::*;
use crate::features::auth::proof;
use crate::features::scan::Observation;
use crate::ports::{
    BuildFacts, OtaError, OtaWriter, PartitionInfo, RadioDriver, RadioError, Settings,
    SettingsError, SettingsStore, System, TelemetrySink,
};

/// What the fake radio was asked to do, shared so a test can look after the radio moved into the `App`.
#[derive(Default)]
struct RadioLog {
    hops: Vec<(u16, Vec<u8>)>,
    channels: Vec<u8>,
    fail_channel: Option<u8>,
}

#[derive(Default)]
struct FakeRadio(u8, Rc<RefCell<RadioLog>>);
impl RadioDriver for FakeRadio {
    fn channel(&self) -> u8 {
        self.0
    }
    fn set_channel(&mut self, c: u8) -> Result<(), RadioError> {
        if self.1.borrow().fail_channel == Some(c) {
            return Err(RadioError::Driver);
        }
        self.1.borrow_mut().channels.push(c);
        self.0 = c;
        Ok(())
    }
    fn configure_hop(&mut self, dwell_ms: u16, order: &[u8]) {
        self.1.borrow_mut().hops.push((dwell_ms, order.to_vec()));
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
    revision: Rc<RefCell<u32>>,
    erased: Rc<RefCell<u32>>,
}
impl SettingsStore for FakeStore {
    fn load(&mut self) -> Settings {
        self.settings.clone().unwrap_or_default()
    }
    fn save(&mut self, s: &Settings) -> Result<(), SettingsError> {
        self.settings = Some(s.clone());
        *self.revision.borrow_mut() += 1;
        Ok(())
    }
    fn save_revision(&mut self) -> u32 {
        *self.revision.borrow()
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
    fn erase_all(&mut self) -> Result<(), SettingsError> {
        self.settings = None;
        *self.key.borrow_mut() = None;
        *self.revision.borrow_mut() = 0;
        *self.erased.borrow_mut() += 1;
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
/// What the diagnostics report; flip a flag to make a self-test step fail.
struct SysState {
    nvs_ok: bool,
    ble_ok: bool,
    ota_ok: bool,
}

impl Default for SysState {
    fn default() -> Self {
        Self {
            nvs_ok: true,
            ble_ok: true,
            ota_ok: true,
        }
    }
}

struct FakeSystem(Rc<RefCell<bool>>, Rc<RefCell<SysState>>);
impl System for FakeSystem {
    fn nvs_usage(&self) -> (u32, u32) {
        (3000, 21_576)
    }
    fn partition_info(&self) -> PartitionInfo {
        PartitionInfo {
            ota0_active: true,
            ota1_active: false,
            ota0_size: 1_572_864,
            ota1_size: 1_572_864,
        }
    }
    fn build_facts(&self) -> BuildFacts {
        BuildFacts {
            version: "0.1.0",
            git_sha: "a1b2c3d",
            built: "2026-11-18T10:00:00Z",
            ota_key_fp: [0xAB; 16],
            request_conn_params: false,
            coex_debug: false,
            bonding: false,
        }
    }
    fn self_test_nvs(&mut self) -> bool {
        self.1.borrow().nvs_ok
    }
    fn self_test_ble(&self) -> bool {
        self.1.borrow().ble_ok
    }
    fn self_test_ota_slot(&self) -> bool {
        self.1.borrow().ota_ok
    }
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
        FakeSystem(rebooted, Rc::default()),
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
        settings: Some({
            let mut s = Settings::default();
            s.radio.default_channel = 9;
            s.scan.scan_at_boot = true;
            s
        }),
        ..Default::default()
    };
    let mut app = App::new(
        FakeRadio::default(),
        FakeOta,
        FakeSystem(Default::default(), Rc::default()),
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
    app.on_deauth_window(&[([1; 6], 9), ([2; 6], 12)]);
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
    app.on_deauth_window(&[([2; 6], 5000)]);
    assert!(results(&sink).is_empty(), "0 = off");
    set_monitor(&mut app, 3, false, 3);
    sink.got.borrow_mut().clear();
    app.on_deauth_window(&[([2; 6], 3)]);
    assert_eq!(results(&sink).len(), 1);
}

#[test]
fn deauth_alerts_are_withheld_until_authenticated() {
    let sink = FakeSink::default();
    let mut app = app_with(sink.clone(), Rc::default(), Some(KEY));
    app.on_deauth_window(&[([2; 6], 500)]);
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

// ---- settings management ------------------------------------------------------------------------

use beewave_protocol::{HopOrder, RejectReason, SETTINGS_SCHEMA};

/// An authenticated app with handles on what the fakes record.
struct Rig {
    app: TestApp,
    sink: FakeSink,
    radio: Rc<RefCell<RadioLog>>,
    sys: Rc<RefCell<SysState>>,
    rebooted: Rc<RefCell<bool>>,
    revision: Rc<RefCell<u32>>,
    erased: Rc<RefCell<u32>>,
}

fn rig(ota_key: Option<[u8; 32]>) -> Rig {
    let sink = FakeSink::default();
    let rebooted = Rc::new(RefCell::new(false));
    let radio: Rc<RefCell<RadioLog>> = Rc::default();
    let sys: Rc<RefCell<SysState>> = Rc::default();
    let store = FakeStore::default();
    let (revision, erased) = (store.revision.clone(), store.erased.clone());
    let mut app = App::new(
        FakeRadio(0, radio.clone()),
        FakeOta,
        FakeSystem(rebooted.clone(), sys.clone()),
        sink.clone(),
        store,
        DeviceKeys {
            auth: Some(KEY),
            ota_signing: ota_key,
        },
    );
    app.start();
    authenticate(&mut app, &sink);
    Rig {
        app,
        sink,
        radio,
        sys,
        rebooted,
        revision,
        erased,
    }
}

fn save(app: &mut TestApp, f: impl FnOnce(&mut Settings)) {
    let mut s = Settings::default();
    f(&mut s);
    app.on_command(DeviceCommand::SaveSettingsAll {
        scan: s.scan,
        radio: s.radio,
        telemetry: s.telemetry,
        alerts: s.alerts,
    });
}

fn last(sink: &FakeSink) -> TelemetryMessage {
    sink.got.borrow().last().cloned().expect("a message")
}

#[test]
fn get_settings_all_reports_the_defaults_with_schema_and_revision() {
    let mut r = rig(None);
    r.app.on_command(DeviceCommand::GetSettingsAll);
    match last(&r.sink) {
        TelemetryMessage::SettingsAll {
            schema,
            revision,
            scan,
            radio,
            telemetry,
            alerts,
        } => {
            assert_eq!((schema, revision), (SETTINGS_SCHEMA, 0));
            let d = Settings::default();
            assert_eq!(
                (scan, radio, telemetry, alerts),
                (d.scan, d.radio, d.telemetry, d.alerts)
            );
        }
        other => panic!("expected SettingsAll, got {other:?}"),
    }
}

#[test]
fn a_valid_save_is_acknowledged_with_the_new_revision_and_read_back() {
    let mut r = rig(None);
    save(&mut r.app, |s| s.scan.dedup_rssi_db = 12);
    assert_eq!(
        last(&r.sink),
        TelemetryMessage::SettingsSaved {
            revision: 1,
            ok: true,
            reason: None
        }
    );
    save(&mut r.app, |s| s.scan.dedup_rssi_db = 13);
    assert!(matches!(
        last(&r.sink),
        TelemetryMessage::SettingsSaved {
            revision: 2,
            ok: true,
            ..
        }
    ));
    r.app.on_command(DeviceCommand::GetSettingsAll);
    assert!(
        matches!(last(&r.sink), TelemetryMessage::SettingsAll { revision: 2, scan, .. } if scan.dedup_rssi_db == 13)
    );
}

#[test]
fn a_rejected_save_names_the_reason_and_leaves_the_stored_settings_alone() {
    let mut r = rig(None);
    save(&mut r.app, |s| s.scan.dedup_rssi_db = 12);
    let before = *r.revision.borrow();
    save(&mut r.app, |s| s.alerts.deauth_window_ms = 100); // 2 x 250 ms interval does not fit
    assert!(matches!(
        last(&r.sink),
        TelemetryMessage::SettingsSaved { ok: false, reason: Some(RejectReason::DeauthWindowRange), revision } if revision == before
    ));
    save(&mut r.app, |s| {
        s.scan.scan_interval_ms = 600;
        s.alerts.deauth_window_ms = 1000;
    });
    assert!(matches!(
        last(&r.sink),
        TelemetryMessage::SettingsSaved {
            ok: false,
            reason: Some(RejectReason::WindowTooShort),
            ..
        }
    ));
    assert_eq!(*r.revision.borrow(), before, "NVS was not written");
    r.app.on_command(DeviceCommand::GetSettingsAll);
    assert!(
        matches!(last(&r.sink), TelemetryMessage::SettingsAll { scan, .. } if scan.dedup_rssi_db == 12)
    );
}

#[test]
fn settings_commands_need_an_authenticated_session() {
    let sink = FakeSink::default();
    let mut app = app_with(sink.clone(), Rc::default(), Some(KEY));
    app.on_command(DeviceCommand::GetSettingsAll);
    save(&mut app, |s| s.scan.dedup_rssi_db = 12);
    app.on_command(DeviceCommand::FactoryReset);
    app.on_command(DeviceCommand::RunSelfTest);
    app.on_command(DeviceCommand::GetStorageInfo);
    app.on_command(DeviceCommand::GetDeviceInfo);
    let got = sink.got.borrow();
    assert_eq!(got.len(), 6);
    assert!(got
        .iter()
        .all(|m| matches!(m, TelemetryMessage::Event { code: 4, .. })));
}

#[test]
fn saved_scan_settings_take_effect_on_the_next_beacons() {
    let mut r = rig(None);
    save(&mut r.app, |s| {
        s.scan.ssid_filter = "caf".try_into().unwrap();
        s.scan.channel_filter_mask = 1 << 5; // channel 6 only
    });
    r.sink.got.borrow_mut().clear();
    let beacon = |id: u8, ssid: &str, ch: u8| Observation {
        bssid: [id; 6],
        channel: ch,
        rssi: -50,
        ssid: ssid.try_into().unwrap(),
    };
    r.app.on_observation(beacon(1, "Cafe", 6));
    r.app.on_observation(beacon(2, "Cafe", 1)); // wrong channel
    r.app.on_observation(beacon(3, "Home", 6)); // wrong name
    let reported: Vec<_> = results(&r.sink)
        .into_iter()
        .filter_map(|m| match m {
            TelemetryMessage::ScanResult { bssid, .. } => Some(bssid),
            _ => None,
        })
        .collect();
    assert_eq!(reported, [[1; 6]]);
}

#[test]
fn hop_settings_reach_the_radio_when_a_scan_starts_not_before() {
    let mut r = rig(None);
    save(&mut r.app, |s| {
        s.radio.hop_dwell_ms = 400;
        s.radio.hop_order = HopOrder::Ascending1_6_11;
    });
    assert!(r.radio.borrow().hops.is_empty(), "nothing mid-hop");
    r.app.on_command(DeviceCommand::StartScan { band_mask: 1 });
    let hops = r.radio.borrow().hops.clone();
    assert_eq!(hops.len(), 1);
    assert_eq!(hops[0].0, 400);
    assert_eq!(hops[0].1, [1, 6, 11, 2, 7, 12, 3, 8, 13, 4, 9, 5, 10]);

    save(&mut r.app, |s| {
        s.radio.hop_order = HopOrder::Custom;
        s.radio.custom_hop_list = [11, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    });
    r.app.on_command(DeviceCommand::StartScan { band_mask: 1 });
    assert_eq!(r.radio.borrow().hops[1].1, [11, 1]);
}

#[test]
fn the_default_channel_is_applied_when_saved() {
    let mut r = rig(None);
    save(&mut r.app, |s| s.radio.default_channel = 9);
    assert_eq!(r.radio.borrow().channels.last(), Some(&9));
}

#[test]
fn the_status_period_and_deauth_window_follow_the_settings() {
    let mut r = rig(None);
    assert_eq!(
        (r.app.status_period_ms(), r.app.deauth_window_ms()),
        (1000, 1000)
    );
    save(&mut r.app, |s| {
        s.telemetry.status_period_ms = 2500;
        s.alerts.deauth_window_ms = 2000;
    });
    assert_eq!(
        (r.app.status_period_ms(), r.app.deauth_window_ms()),
        (2500, 2000)
    );
    // 19 frames in 2 s is 9 per second (below 10): no alert; 20 is
    r.sink.got.borrow_mut().clear();
    r.app.on_deauth_window(&[([1; 6], 19)]);
    assert!(results(&r.sink).is_empty());
    r.app.on_deauth_window(&[([1; 6], 20)]);
    assert_eq!(
        results(&r.sink),
        [TelemetryMessage::DeauthAlert {
            bssid: [1; 6],
            count: 10
        }]
    );
}

#[test]
fn a_saved_deauth_threshold_becomes_the_active_one() {
    let mut r = rig(None);
    save(&mut r.app, |s| s.alerts.deauth_threshold = 3);
    r.app.on_command(DeviceCommand::GetMonitor);
    assert!(matches!(
        last(&r.sink),
        TelemetryMessage::MonitorConfig {
            deauth_threshold: 3,
            ..
        }
    ));
}

#[test]
fn factory_reset_erases_everything_and_reboots() {
    let mut r = rig(None);
    save(&mut r.app, |s| s.scan.dedup_rssi_db = 12);
    r.app.on_command(DeviceCommand::FactoryReset);
    assert!(*r.rebooted.borrow());
    assert_eq!(*r.erased.borrow(), 1);
    assert_eq!(*r.revision.borrow(), 0);
}

#[test]
fn factory_reset_is_refused_while_an_update_is_in_flight() {
    let mut r = rig(Some([9; 32]));
    r.app.on_command(DeviceCommand::OtaBegin {
        size: 1000,
        sha256: [0; 32],
    });
    r.sink.got.borrow_mut().clear();
    r.app.on_command(DeviceCommand::FactoryReset);
    assert_eq!(
        results(&r.sink),
        [TelemetryMessage::Rejected {
            reason: RejectReason::Busy
        }]
    );
    assert!(!*r.rebooted.borrow());
    assert_eq!(*r.erased.borrow(), 0);
}

#[test]
fn self_test_reports_each_step_and_restores_the_pinned_channel() {
    let mut r = rig(None);
    save(&mut r.app, |s| s.radio.default_channel = 6);
    r.app.on_command(DeviceCommand::RunSelfTest);
    assert_eq!(
        last(&r.sink),
        TelemetryMessage::SelfTestResult {
            radio: true,
            nvs: true,
            ble: true,
            ota_slot: true,
            busy: false
        }
    );
    let ch = r.radio.borrow().channels.clone();
    assert_eq!(
        &ch[ch.len() - 3..],
        [1, 13, 6],
        "1, then 13, then back to the pinned channel"
    );

    r.sys.borrow_mut().nvs_ok = false;
    r.sys.borrow_mut().ota_ok = false;
    r.radio.borrow_mut().fail_channel = Some(13);
    r.app.on_command(DeviceCommand::RunSelfTest);
    assert_eq!(
        last(&r.sink),
        TelemetryMessage::SelfTestResult {
            radio: false,
            nvs: false,
            ble: true,
            ota_slot: false,
            busy: false
        }
    );
}

#[test]
fn self_test_does_not_touch_the_radio_while_scanning() {
    let mut r = rig(None);
    r.app.on_command(DeviceCommand::StartScan { band_mask: 1 });
    let calls = r.radio.borrow().channels.len();
    r.app.on_command(DeviceCommand::RunSelfTest);
    assert_eq!(
        last(&r.sink),
        TelemetryMessage::SelfTestResult {
            radio: false,
            nvs: false,
            ble: false,
            ota_slot: false,
            busy: true
        }
    );
    assert_eq!(r.radio.borrow().channels.len(), calls);
}

#[test]
fn storage_and_device_info_come_from_the_system_port() {
    let mut r = rig(None);
    save(&mut r.app, |s| s.scan.dedup_rssi_db = 12);
    r.app.on_command(DeviceCommand::GetStorageInfo);
    assert_eq!(
        last(&r.sink),
        TelemetryMessage::StorageInfo {
            nvs_used: 3000,
            nvs_free: 21_576,
            ota0_active: true,
            ota1_active: false,
            ota0_size: 1_572_864,
            ota1_size: 1_572_864,
            settings_revision: 1,
        }
    );
    r.app.on_command(DeviceCommand::GetDeviceInfo);
    match last(&r.sink) {
        TelemetryMessage::DeviceInfo {
            version,
            git_sha,
            ota_key_fp,
            request_conn_params,
            bonding,
            ..
        } => {
            assert_eq!((version.as_str(), git_sha.as_str()), ("0.1.0", "a1b2c3d"));
            assert_eq!(ota_key_fp, [0xAB; 16]);
            assert!(!request_conn_params && !bonding);
        }
        other => panic!("expected DeviceInfo, got {other:?}"),
    }
}

#[test]
fn the_legacy_settings_commands_still_work() {
    let mut r = rig(None);
    r.app.on_command(DeviceCommand::SetSettings {
        channel: 6,
        auto_scan: true,
    });
    assert_eq!(
        last(&r.sink),
        TelemetryMessage::Settings {
            channel: 6,
            auto_scan: true
        }
    );
    r.app.on_command(DeviceCommand::GetSettings);
    assert_eq!(
        last(&r.sink),
        TelemetryMessage::Settings {
            channel: 6,
            auto_scan: true
        }
    );
    r.app.on_command(DeviceCommand::SetSettings {
        channel: 99,
        auto_scan: true,
    });
    assert!(matches!(
        last(&r.sink),
        TelemetryMessage::Event { code: 3, .. }
    ));
}

// ---- passive handshake capture ------------------------------------------------------------------

use crate::features::handshake::parse_eapol;

/// A minimal 802.11 data frame carrying one EAPOL-Key message with the given key-info bits.
fn eapol_frame(key_info: u16, from_ds: bool, bssid: [u8; 6], client: [u8; 6]) -> std::vec::Vec<u8> {
    let mut f = std::vec::Vec::new();
    let fc1 = if from_ds { 0x02 } else { 0x01 };
    f.extend_from_slice(&[0x08, fc1]);
    f.extend_from_slice(&[0; 2]);
    let (addr1, addr2) = if from_ds {
        (client, bssid)
    } else {
        (bssid, client)
    };
    f.extend_from_slice(&addr1);
    f.extend_from_slice(&addr2);
    f.extend_from_slice(&bssid);
    f.extend_from_slice(&[0; 2]);
    f.extend_from_slice(&[0xAA, 0xAA, 0x03, 0x00, 0x00, 0x00, 0x88, 0x8E]);
    f.extend_from_slice(&[1, 3, 0, 0]);
    f.push(2);
    f.extend_from_slice(&key_info.to_be_bytes());
    f.extend_from_slice(&[0; 2]);
    f.extend_from_slice(&[0u8; 8]); // replay counter
    f.extend_from_slice(&[0x11; 32]); // nonce
    f.extend_from_slice(&[0; 16]);
    f.extend_from_slice(&[0; 8]);
    f.extend_from_slice(&[0; 8]);
    f.extend_from_slice(&[0x22; 16]); // MIC
    f.extend_from_slice(&[0; 2]); // key data length
    f
}

const PAIRWISE: u16 = 1 << 3;
const MSG1: u16 = PAIRWISE | (1 << 7);
const MSG2: u16 = PAIRWISE | (1 << 8);

#[test]
fn handshake_capture_is_off_by_default_and_toggles() {
    let mut r = rig(None);
    r.app.on_command(DeviceCommand::GetHandshakeCapture);
    assert_eq!(
        last(&r.sink),
        TelemetryMessage::HandshakeCaptureState { enabled: false }
    );
    r.app
        .on_command(DeviceCommand::SetHandshakeCapture { enabled: true });
    assert_eq!(
        last(&r.sink),
        TelemetryMessage::HandshakeCaptureState { enabled: true }
    );
    assert!(r.app.handshake_capture_enabled());
}

#[test]
fn handshake_commands_need_an_authenticated_session() {
    let sink = FakeSink::default();
    let mut app = app_with(sink.clone(), Rc::default(), Some(KEY));
    app.on_command(DeviceCommand::GetHandshakeCapture);
    app.on_command(DeviceCommand::SetHandshakeCapture { enabled: true });
    let got = sink.got.borrow();
    assert_eq!(got.len(), 2);
    assert!(got
        .iter()
        .all(|m| matches!(m, TelemetryMessage::Event { code: 4, .. })));
}

#[test]
fn eapol_frames_are_ignored_while_the_listener_is_off() {
    let mut r = rig(None);
    let f = parse_eapol(&eapol_frame(MSG1, true, [1; 6], [2; 6])).unwrap();
    r.app.on_eapol_frame(f);
    assert!(results(&r.sink).is_empty());
}

#[test]
fn a_classified_frame_is_reported_and_completion_fires_once() {
    let mut r = rig(None);
    r.app
        .on_command(DeviceCommand::SetHandshakeCapture { enabled: true });
    r.sink.got.borrow_mut().clear();

    let m1 = parse_eapol(&eapol_frame(MSG1, true, [1; 6], [2; 6])).unwrap();
    r.app.on_eapol_frame(m1);
    assert!(matches!(
        results(&r.sink)[0],
        TelemetryMessage::HandshakeFrame { message: 1, .. }
    ));

    r.sink.got.borrow_mut().clear();
    let m2 = parse_eapol(&eapol_frame(MSG2, false, [1; 6], [2; 6])).unwrap();
    r.app.on_eapol_frame(m2);
    let got = results(&r.sink);
    assert!(matches!(
        got[0],
        TelemetryMessage::HandshakeFrame { message: 2, .. }
    ));
    assert_eq!(
        got[1],
        TelemetryMessage::HandshakeComplete {
            bssid: [1; 6],
            client: [2; 6]
        }
    );
}

#[test]
fn switching_the_listener_back_on_forgets_earlier_progress() {
    let mut r = rig(None);
    r.app
        .on_command(DeviceCommand::SetHandshakeCapture { enabled: true });
    r.app
        .on_eapol_frame(parse_eapol(&eapol_frame(MSG1, true, [1; 6], [2; 6])).unwrap());
    r.app
        .on_command(DeviceCommand::SetHandshakeCapture { enabled: false });
    r.app
        .on_command(DeviceCommand::SetHandshakeCapture { enabled: true });
    r.sink.got.borrow_mut().clear();
    r.app
        .on_eapol_frame(parse_eapol(&eapol_frame(MSG2, false, [1; 6], [2; 6])).unwrap());
    // message 2 alone (message 1 was forgotten on the reset) must not complete the pair
    assert!(!results(&r.sink)
        .iter()
        .any(|m| matches!(m, TelemetryMessage::HandshakeComplete { .. })));
}

#[test]
fn routes_probe_observations_with_deduplication() {
    let mut r = rig(None);
    let probe = crate::features::probe::ProbeObservation {
        client: [0x11, 0x22, 0x33, 0x44, 0x55, 0x66],
        channel: 6,
        rssi: -60,
        ssid: heapless::String::try_from("GuestWiFi").unwrap(),
    };
    r.app.on_probe(probe.clone());
    let got = results(&r.sink);
    assert_eq!(got.len(), 1);
    assert_eq!(
        got[0],
        TelemetryMessage::ProbeResult {
            client: [0x11, 0x22, 0x33, 0x44, 0x55, 0x66],
            channel: 6,
            rssi: -60,
            ssid: heapless::String::try_from("GuestWiFi").unwrap(),
        }
    );

    // Duplicate probe is suppressed
    r.sink.got.borrow_mut().clear();
    r.app.on_probe(probe);
    assert!(results(&r.sink).is_empty());
}

#[test]
fn routes_assoc_observations_with_deduplication() {
    let mut r = rig(None);
    let assoc = crate::features::assoc::AssocObservation {
        bssid: [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
        client: [0x00, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE],
        channel: 6,
        rssi: -55,
    };
    r.app.on_assoc(assoc);
    let got = results(&r.sink);
    assert_eq!(got.len(), 1);
    assert_eq!(
        got[0],
        TelemetryMessage::AssocResult {
            bssid: [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
            client: [0x00, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE],
            channel: 6,
            rssi: -55,
        }
    );

    // Duplicate association is suppressed
    r.sink.got.borrow_mut().clear();
    r.app.on_assoc(assoc);
    assert!(results(&r.sink).is_empty());
}
