use walpad_protocol::DeviceCommand;

use super::types::{Mode, MAX_CHANNEL};
use crate::ports::{RadioDriver, RadioError};

/// Owns the radio driver `R` and the current mode/channel.
/// `R: RadioDriver` is a generic bound: any type implementing the port can be plugged in.
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

    /// `None` means "not a radio command, someone else handles it".
    /// `Some(Ok(()))` / `Some(Err(e))` is the outcome of a radio command.
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
                    return Some(Err(RadioError::InvalidChannel)); // rejected before touching hardware
                }
                self.driver
                    .set_channel(*channel)
                    .map(|_| self.channel = *channel)
            }
            _ => return None,
        })
    }
}
