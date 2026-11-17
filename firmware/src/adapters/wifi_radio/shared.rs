use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU8, Ordering};

use super::hopper::{DWELL, HOP_ORDER};

/// State shared by the driver (commands) and the hopper thread. Plain atomics, no locks.
pub struct Shared {
    /// Promiscuous mode on?
    pub sniffing: AtomicBool,
    /// 0 = hop across all channels, 1..=13 = stay on that channel.
    pub pinned: AtomicU8,
    /// Channel the radio is on right now.
    pub current: AtomicU8,
    /// Time on each channel while hopping (ms). Set from the owner's settings when a scan starts.
    pub dwell_ms: AtomicU16,
    /// The channels to visit while hopping, in order (`order_len` of them).
    order: [AtomicU8; HOP_ORDER.len()],
    order_len: AtomicU8,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            sniffing: AtomicBool::new(false),
            pinned: AtomicU8::new(0),
            current: AtomicU8::new(1),
            dwell_ms: AtomicU16::new(DWELL.as_millis() as u16),
            order: HOP_ORDER.map(AtomicU8::new),
            order_len: AtomicU8::new(HOP_ORDER.len() as u8),
        }
    }

    /// Adopt a hop order (the first `HOP_ORDER.len()` entries are used; an empty list is ignored).
    pub fn set_order(&self, order: &[u8]) {
        let n = order.len().min(self.order.len());
        if n == 0 {
            return;
        }
        for (slot, &c) in self.order.iter().zip(order) {
            slot.store(c, Ordering::Relaxed);
        }
        self.order_len.store(n as u8, Ordering::Relaxed);
    }

    /// The current hop order copied out (the hopper walks this copy for one step).
    pub fn order(&self) -> ([u8; HOP_ORDER.len()], usize) {
        let mut out = [0u8; HOP_ORDER.len()];
        let n = usize::from(self.order_len.load(Ordering::Relaxed)).min(out.len());
        for (o, slot) in out.iter_mut().zip(&self.order).take(n) {
            *o = slot.load(Ordering::Relaxed);
        }
        (out, n)
    }
}
