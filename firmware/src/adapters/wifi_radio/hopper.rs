use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use beewave_core::features::radio::{next_in_order, MAX_CHANNEL};
use esp_idf_svc::sys::{self, esp, EspError};

use super::shared::Shared;

// ---- Tuning knobs (channel hopping) -----------------------------------------------------------
/// How long to listen on each channel while hopping. Shorter = fresher results but more radio
/// time lost to switching; the Wi-Fi radio also shares airtime with BLE.
pub const DWELL: Duration = Duration::from_millis(250);

/// The order channels are visited in. Sequential is the default; the commented alternative visits
/// the non-overlapping channels (1, 6, 11) first, which may find busy networks sooner:
/// `[1, 6, 11, 2, 7, 12, 3, 8, 13, 4, 9, 5, 10]`. Must contain every channel 1..=13 exactly once.
pub const HOP_ORDER: [u8; MAX_CHANNEL as usize] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

/// Tune the radio. Thread-safe on the ESP-IDF side.
pub fn set_channel(channel: u8) -> Result<(), EspError> {
    // SAFETY: plain driver call; `channel` was validated by the core (1..=13).
    esp!(unsafe {
        sys::esp_wifi_set_channel(channel, sys::wifi_second_chan_t_WIFI_SECOND_CHAN_NONE)
    })
}

/// Long-running thread: while sniffing and not pinned, step through `HOP_ORDER`.
pub fn run(shared: Arc<Shared>) {
    loop {
        std::thread::sleep(DWELL);
        if !shared.sniffing.load(Ordering::Relaxed) || shared.pinned.load(Ordering::Relaxed) != 0 {
            continue;
        }
        let next = next_in_order(&HOP_ORDER, shared.current.load(Ordering::Relaxed));
        if set_channel(next).is_ok() {
            shared.current.store(next, Ordering::Relaxed);
        }
    }
}
