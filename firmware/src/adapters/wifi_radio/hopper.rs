use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use beewave_core::features::radio::MAX_CHANNEL;
use esp_idf_svc::sys::{self, esp, EspError};

use super::shared::Shared;

/// How long to listen on each channel while hopping.
const DWELL: Duration = Duration::from_millis(250);

/// Tune the radio. Thread-safe on the ESP-IDF side.
pub fn set_channel(channel: u8) -> Result<(), EspError> {
    // SAFETY: plain driver call; `channel` was validated by the core (1..=13).
    esp!(unsafe {
        sys::esp_wifi_set_channel(channel, sys::wifi_second_chan_t_WIFI_SECOND_CHAN_NONE)
    })
}

/// Long-running thread: while sniffing and not pinned, step through channels 1..=13.
pub fn run(shared: Arc<Shared>) {
    loop {
        std::thread::sleep(DWELL);
        if !shared.sniffing.load(Ordering::Relaxed) || shared.pinned.load(Ordering::Relaxed) != 0 {
            continue;
        }
        let next = shared.current.load(Ordering::Relaxed) % MAX_CHANNEL + 1;
        if set_channel(next).is_ok() {
            shared.current.store(next, Ordering::Relaxed);
        }
    }
}
