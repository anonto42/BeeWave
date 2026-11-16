use walpad_protocol::DeviceCommand;

use super::*;
use crate::ports::{RadioDriver, RadioError};

/// A fake driver: records what the controller asked for instead of touching hardware.
#[derive(Default)]
struct FakeRadio {
    sniffing: bool,
    channel: u8,
}

impl RadioDriver for FakeRadio {
    fn set_channel(&mut self, c: u8) -> Result<(), RadioError> {
        self.channel = c;
        Ok(())
    }
    fn start_sniffing(&mut self) -> Result<(), RadioError> {
        self.sniffing = true;
        Ok(())
    }
    fn stop_sniffing(&mut self) -> Result<(), RadioError> {
        self.sniffing = false;
        Ok(())
    }
    fn channel(&self) -> u8 {
        self.channel
    }
}

#[test]
fn scan_and_channel() {
    let mut r = RadioController::new(FakeRadio::default());
    assert_eq!(
        r.handle(&DeviceCommand::StartScan { band_mask: 1 }),
        Some(Ok(()))
    );
    assert_eq!(r.mode(), Mode::Scanning);
    assert_eq!(
        r.handle(&DeviceCommand::ConfigureRadio { channel: 99 }),
        Some(Err(RadioError::InvalidChannel))
    );
    assert_eq!(
        r.handle(&DeviceCommand::ConfigureRadio { channel: 6 }),
        Some(Ok(()))
    );
    assert_eq!(r.channel(), 6);
    assert_eq!(r.handle(&DeviceCommand::RebootDevice), None);
    r.handle(&DeviceCommand::StopOperation);
    assert_eq!(r.mode(), Mode::Idle);
}

#[test]
fn auto_channel_is_accepted() {
    let mut r = RadioController::new(FakeRadio::default());
    assert_eq!(
        r.handle(&DeviceCommand::ConfigureRadio {
            channel: AUTO_CHANNEL
        }),
        Some(Ok(()))
    );
}
