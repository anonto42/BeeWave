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
