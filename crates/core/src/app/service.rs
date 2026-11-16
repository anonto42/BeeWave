use walpad_protocol::{DeviceCommand, TelemetryMessage};

use super::messages::{event, radio_error_text, EVENT_OTA_ERROR, EVENT_RADIO_ERROR};
use crate::features::ota::OtaSession;
use crate::features::radio::RadioController;
use crate::features::scan::{Observation, ScanTable};
use crate::features::status;
use crate::ports::{OtaWriter, RadioDriver, System, TelemetrySink};

/// Composes the feature slices. The four generic parameters are the four driven ports;
/// the firmware plugs in real adapters, tests plug in fakes.
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

    /// Messages the sink refused since boot (reported in `Status.dropped`).
    pub fn dropped(&self) -> u16 {
        self.dropped
    }

    /// Driving port: a command arrived from the client.
    pub fn on_command(&mut self, cmd: DeviceCommand) {
        if matches!(cmd, DeviceCommand::StartScan { .. }) {
            self.scans.clear();
        }
        if matches!(cmd, DeviceCommand::RebootDevice) {
            self.sys.restart();
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
                Err(_) => self.emit(event(EVENT_OTA_ERROR, "ota error")),
            }
        }
    }

    /// Driving port: the sniffer saw a frame.
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

    fn emit(&mut self, m: TelemetryMessage) {
        if !self.sink.publish(m) {
            self.dropped = self.dropped.saturating_add(1);
        }
    }
}
