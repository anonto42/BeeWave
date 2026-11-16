use beewave_protocol::DeviceCommand;

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

#[test]
fn channel_range_edges() {
    let mut r = RadioController::new(FakeRadio::default());
    let set = |r: &mut RadioController<FakeRadio>, channel| {
        r.handle(&DeviceCommand::ConfigureRadio { channel })
    };
    for ok in [0, 1, 13] {
        assert_eq!(set(&mut r, ok), Some(Ok(())), "channel {ok}");
    }
    for bad in [14, 15, 255] {
        assert_eq!(
            set(&mut r, bad),
            Some(Err(RadioError::InvalidChannel)),
            "channel {bad}"
        );
    }
    assert_eq!(r.channel(), 13, "a rejected channel leaves the radio alone");
}

const SEQUENTIAL: [u8; 13] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
/// Non-overlapping channels first (1, 6, 11), then the rest.
const SPREAD: [u8; 13] = [1, 6, 11, 2, 7, 12, 3, 8, 13, 4, 9, 5, 10];

#[test]
fn hop_follows_the_order_and_wraps() {
    assert_eq!(next_in_order(&SEQUENTIAL, 1), 2);
    assert_eq!(next_in_order(&SEQUENTIAL, 13), 1);
    assert_eq!(next_in_order(&SPREAD, 1), 6);
    assert_eq!(next_in_order(&SPREAD, 10), 1);
}

#[test]
fn hop_visits_every_channel_once_per_cycle() {
    for order in [&SEQUENTIAL, &SPREAD] {
        let mut seen = [false; 14];
        let mut c = order[0];
        for _ in 0..order.len() {
            seen[c as usize] = true;
            c = next_in_order(order, c);
        }
        assert_eq!(c, order[0], "returns to the start after one cycle");
        assert!((1..=13).all(|ch| seen[ch]));
    }
}

#[test]
fn hop_restarts_when_current_is_not_in_the_order() {
    assert_eq!(next_in_order(&SPREAD, 0), 1);
    assert_eq!(next_in_order(&SPREAD, 99), 1);
    assert_eq!(next_in_order(&[], 5), 5);
}
