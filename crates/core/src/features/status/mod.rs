//! Status feature: periodic device health sample.
use crate::ports::System;
use walpad_protocol::TelemetryMessage;

pub fn sample<S: System>(sys: &S, channel: u8, dropped: u16) -> TelemetryMessage {
    TelemetryMessage::Status {
        active_channel: channel,
        free_heap: sys.free_heap(),
        cpu_load: sys.cpu_load(),
        dropped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Sys;
    impl System for Sys {
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
            sample(&Sys, 3, 2),
            TelemetryMessage::Status {
                active_channel: 3,
                free_heap: 1234,
                cpu_load: 7,
                dropped: 2
            }
        );
    }
}
