use beewave_protocol::TelemetryMessage;

use super::*;
use crate::ports::System;

struct FakeSystem;

impl System for FakeSystem {
    fn free_heap(&self) -> u32 {
        1234
    }
    fn cpu_load(&self) -> u8 {
        7
    }
    fn restart(&mut self) {}
}

#[test]
fn builds_status() {
    assert_eq!(
        sample(&FakeSystem, 3, 2),
        TelemetryMessage::Status {
            active_channel: 3,
            free_heap: 1234,
            cpu_load: 7,
            dropped: 2
        }
    );
}
