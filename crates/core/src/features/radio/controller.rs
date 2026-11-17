use beewave_protocol::DeviceCommand;

use super::types::{Mode, AUTO_CHANNEL, MAX_CHANNEL};
use crate::ports::{RadioDriver, RadioError};

/// Owns the radio driver `R` and the current mode.
/// `R: RadioDriver` is a generic bound: any type implementing the port can be plugged in.
pub struct RadioController<R: RadioDriver> {
    driver: R,
    mode: Mode,
    /// The channel the owner pinned (0 = AUTO), restored after the self-test.
    pinned: u8,
}

impl<R: RadioDriver> RadioController<R> {
    pub fn new(driver: R) -> Self {
        Self {
            driver,
            mode: Mode::Idle,
            pinned: AUTO_CHANNEL,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// The channel the radio is on right now (the driver knows, it may be hopping).
    pub fn channel(&self) -> u8 {
        self.driver.channel()
    }

    /// How to hop the next time scanning starts (applied by the driver at that moment, never mid-hop).
    pub fn configure_hop(&mut self, dwell_ms: u16, order: &[u8]) {
        self.driver.configure_hop(dwell_ms, order);
    }

    /// Self-test: tune to channel 1 and 13, then restore the pinned channel. False if any step fails.
    /// The caller must not run it while scanning.
    pub fn self_test(&mut self) -> bool {
        let ok = self.driver.set_channel(1).is_ok() && self.driver.set_channel(13).is_ok();
        let restored = self.driver.set_channel(self.pinned).is_ok();
        ok && restored
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
                if *channel != AUTO_CHANNEL && !(1..=MAX_CHANNEL).contains(channel) {
                    return Some(Err(RadioError::InvalidChannel)); // rejected before touching hardware
                }
                self.driver
                    .set_channel(*channel)
                    .map(|_| self.pinned = *channel)
            }
            _ => return None,
        })
    }
}
