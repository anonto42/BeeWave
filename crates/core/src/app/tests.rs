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
struct FakeStore(Option<Settings>);
impl SettingsStore for FakeStore {
    fn load(&mut self) -> Settings {
        self.0.unwrap_or_default()
    }
    fn save(&mut self, s: &Settings) -> Result<(), SettingsError> {
        self.0 = Some(*s);
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
    App::new(
        FakeRadio::default(),
        FakeOta,
        FakeSystem(rebooted),
        sink,
        FakeStore::default(),
        key,
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
    let store = FakeStore(Some(Settings {
        channel: 9,
        auto_scan: true,
    }));
    let mut app = App::new(
        FakeRadio::default(),
        FakeOta,
        FakeSystem(Default::default()),
        sink.clone(),
        store,
        Some(KEY),
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
fn device_without_a_key_stays_locked() {
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
