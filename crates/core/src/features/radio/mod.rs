//! Radio control feature: scan on/off + channel selection with validation.
use crate::ports::{RadioDriver, RadioError};
use walpad_protocol::DeviceCommand;

pub const MAX_CHANNEL: u8 = 13;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Idle,
    Scanning,
}

pub struct RadioController<R: RadioDriver> {
    driver: R,
    mode: Mode,
    channel: u8,
}

impl<R: RadioDriver> RadioController<R> {
    pub fn new(driver: R) -> Self {
        Self {
            driver,
            mode: Mode::Idle,
            channel: 1,
        }
    }
    pub fn mode(&self) -> Mode {
        self.mode
    }
    pub fn channel(&self) -> u8 {
        self.channel
    }

    /// Returns None if the command isn't ours.
    pub fn handle(&mut self, cmd: &DeviceCommand) -> Option<Result<(), RadioError>> {
        Some(match cmd {
            DeviceCommand::StartScan { .. } => self
                .driver
                .start_sniffing()
                .map(|_| self.mode = Mode::Scanning),
            DeviceCommand::StopOperation => {
                self.driver.stop_sniffing().map(|_| self.mode = Mode::Idle)
            }
            DeviceCommand::ConfigureRadio { channel } => {
                if !(1..=MAX_CHANNEL).contains(channel) {
                    return Some(Err(RadioError::InvalidChannel));
                }
                self.driver
                    .set_channel(*channel)
                    .map(|_| self.channel = *channel)
            }
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Fake {
        sniffing: bool,
        ch: u8,
    }
    impl RadioDriver for Fake {
        fn set_channel(&mut self, c: u8) -> Result<(), RadioError> {
            self.ch = c;
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
    }
    #[test]
    fn scan_and_channel() {
        let mut r = RadioController::new(Fake::default());
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
}
