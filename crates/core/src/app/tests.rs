extern crate std;
use std::{cell::RefCell, rc::Rc, vec::Vec};

use walpad_protocol::{DeviceCommand, TelemetryMessage};

use super::*;
use crate::features::scan::Observation;
use crate::ports::{OtaError, OtaWriter, RadioDriver, RadioError, System, TelemetrySink};

struct FakeRadio;
impl RadioDriver for FakeRadio {
    fn set_channel(&mut self, _: u8) -> Result<(), RadioError> {
        Ok(())
    }
    fn start_sniffing(&mut self) -> Result<(), RadioError> {
        Ok(())
    }
    fn stop_sniffing(&mut self) -> Result<(), RadioError> {
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

#[test]
fn routes_commands_and_observations() {
    let sink = FakeSink::default();
    let rebooted = Rc::new(RefCell::new(false));
    let mut app = App::new(
        FakeRadio,
        FakeOta,
        FakeSystem(rebooted.clone()),
        sink.clone(),
    );

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
    let mut app = App::new(FakeRadio, FakeOta, FakeSystem(Default::default()), sink);
    app.on_tick();
    app.on_tick();
    assert_eq!(app.dropped(), 2);
}
