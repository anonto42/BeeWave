//! Application service: routes driving-adapter input to feature slices and publishes results.
use crate::features::{
    ota::OtaSession,
    radio::RadioController,
    scan::{Observation, ScanTable},
    status,
};
use crate::ports::{OtaWriter, RadioDriver, System, TelemetrySink};
use walpad_protocol::{DeviceCommand, TelemetryMessage};

pub struct App<R: RadioDriver, W: OtaWriter, S: System, T: TelemetrySink> {
    radio: RadioController<R>,
    ota: OtaSession<W>,
    scans: ScanTable,
    sys: S,
    sink: T,
    dropped: u16,
}

impl<R: RadioDriver, W: OtaWriter, S: System, T: TelemetrySink> App<R, W, S, T> {
    pub fn new(radio: R, ota: W, sys: S, sink: T) -> Self {
        Self {
            radio: RadioController::new(radio),
            ota: OtaSession::new(ota),
            scans: ScanTable::new(),
            sys,
            sink,
            dropped: 0,
        }
    }

    fn emit(&mut self, m: TelemetryMessage) {
        if !self.sink.publish(m) {
            self.dropped = self.dropped.saturating_add(1);
        }
    }

    pub fn on_command(&mut self, cmd: DeviceCommand) {
        if matches!(cmd, DeviceCommand::StartScan { .. }) {
            self.scans.clear();
        }
        if matches!(cmd, DeviceCommand::RebootDevice) {
            self.sys.restart();
            return;
        }
        if let Some(r) = self.radio.handle(&cmd) {
            if let Err(e) = r {
                self.emit(event(1, e_str_radio(e)));
            }
            return;
        }
        if let Some(r) = self.ota.handle(&cmd) {
            match r {
                Ok(Some(ack)) => self.emit(ack),
                Ok(None) => {}
                Err(_) => self.emit(event(2, "ota error")),
            }
        }
    }

    /// Driving port: a frame seen by the sniffer.
    pub fn on_observation(&mut self, o: Observation) {
        if let Some(m) = self.scans.observe(o) {
            self.emit(m);
        }
    }

    /// Driving port: periodic tick (status heartbeat).
    pub fn on_tick(&mut self) {
        let m = status::sample(&self.sys, self.radio.channel(), self.dropped);
        self.emit(m);
    }
}

fn e_str_radio(e: crate::ports::RadioError) -> &'static str {
    match e {
        crate::ports::RadioError::InvalidChannel => "invalid channel",
        crate::ports::RadioError::Driver => "radio driver error",
    }
}

fn event(code: u16, msg: &str) -> TelemetryMessage {
    let mut s = heapless::String::new();
    let _ = s.push_str(&msg[..msg.len().min(64)]);
    TelemetryMessage::Event { code, message: s }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{OtaError, RadioError};
    extern crate std;
    use std::{cell::RefCell, rc::Rc, vec::Vec};

    struct R;
    impl RadioDriver for R {
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
    struct W;
    impl OtaWriter for W {
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
    struct S(Rc<RefCell<bool>>);
    impl System for S {
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
    #[derive(Clone, Default)]
    struct Sink(Rc<RefCell<Vec<TelemetryMessage>>>, bool);
    impl TelemetrySink for Sink {
        fn publish(&mut self, m: TelemetryMessage) -> bool {
            if self.1 {
                return false;
            }
            self.0.borrow_mut().push(m);
            true
        }
    }

    #[test]
    fn routes_commands_and_observations() {
        let sink = Sink::default();
        let rebooted = Rc::new(RefCell::new(false));
        let mut app = App::new(R, W, S(rebooted.clone()), sink.clone());
        app.on_command(DeviceCommand::ConfigureRadio { channel: 99 });
        app.on_command(DeviceCommand::ConfigureRadio { channel: 6 });
        app.on_observation(Observation {
            bssid: [1; 6],
            channel: 6,
            rssi: -40,
            ssid: "a".try_into().unwrap(),
        });
        app.on_tick();
        let out = sink.0.borrow();
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
        let mut app = App::new(R, W, S(Default::default()), Sink(Default::default(), true));
        app.on_tick();
        app.on_tick();
        assert_eq!(app.dropped, 2);
    }
}
