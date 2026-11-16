use std::sync::atomic::{AtomicBool, AtomicU8};

/// State shared by the driver (commands) and the hopper thread. Plain atomics, no locks.
pub struct Shared {
    /// Promiscuous mode on?
    pub sniffing: AtomicBool,
    /// 0 = hop across all channels, 1..=13 = stay on that channel.
    pub pinned: AtomicU8,
    /// Channel the radio is on right now.
    pub current: AtomicU8,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            sniffing: AtomicBool::new(false),
            pinned: AtomicU8::new(0),
            current: AtomicU8::new(1),
        }
    }
}
