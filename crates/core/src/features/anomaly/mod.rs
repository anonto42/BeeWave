//! Anomaly detector (receive-only): watches the beacon stream for behaviour that is odd for a Wi-Fi
//! network. It only reads what the sniffer already hears; nothing is transmitted.
mod detector;
mod types;

pub use detector::AnomalyDetector;
pub use types::{
    Anomaly, CAPACITY, COOLDOWN_TICKS, CYCLE_TICKS, LEARN_TICKS, RETURN_AFTER_CYCLES, RSSI_JUMP_DB,
    TWO_CHANNEL_WINDOW_TICKS,
};

#[cfg(test)]
mod tests;
