//! Disassociation-frame counters (receive-only). The promiscuous callback runs in the Wi-Fi driver
//! task and must not block, allocate or queue per frame, so it bumps lock-free counters;
//! the app task drains them once per configured deauth window.
use std::sync::atomic::{AtomicU16, AtomicU32, Ordering};

pub const SLOTS: usize = 8;

const IN_USE: u32 = 1 << 16;

struct Slot {
    hi: AtomicU32,
    lo: AtomicU32,
    count: AtomicU16,
}

static TABLE: [Slot; SLOTS] = [const {
    Slot {
        hi: AtomicU32::new(0),
        lo: AtomicU32::new(0),
        count: AtomicU16::new(0),
    }
}; SLOTS];

pub static OVERFLOW: AtomicU32 = AtomicU32::new(0);

fn split(bssid: &[u8; 6]) -> (u32, u32) {
    (
        u32::from_be_bytes([bssid[0], bssid[1], bssid[2], bssid[3]]),
        IN_USE | u32::from(u16::from_be_bytes([bssid[4], bssid[5]])),
    )
}

fn join(hi: u32, lo: u32) -> [u8; 6] {
    let h = hi.to_be_bytes();
    let l = (lo as u16).to_be_bytes();
    [h[0], h[1], h[2], h[3], l[0], l[1]]
}

/// Count one disassociation frame from `bssid`. Called from the Wi-Fi task: lock-free, no allocation.
pub fn record(bssid: &[u8; 6]) {
    let (hi, lo) = split(bssid);
    for slot in &TABLE {
        let current = slot.lo.load(Ordering::Acquire);
        if current == lo && slot.hi.load(Ordering::Relaxed) == hi {
            bump(slot);
            return;
        }
        if current == 0 {
            slot.hi.store(hi, Ordering::Relaxed);
            slot.lo.store(lo, Ordering::Release);
            bump(slot);
            return;
        }
    }
    OVERFLOW.fetch_add(1, Ordering::Relaxed);
}

fn bump(slot: &Slot) {
    let _ = slot
        .count
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |c| {
            Some(c.saturating_add(1))
        });
}

/// The counts since the last call, reset to zero. Returns how many entries of `out` are filled.
pub fn drain(out: &mut [([u8; 6], u16); SLOTS]) -> usize {
    let mut n = 0;
    for slot in &TABLE {
        let count = slot.count.swap(0, Ordering::Relaxed);
        if count == 0 {
            continue;
        }
        let lo = slot.lo.load(Ordering::Acquire);
        if lo & IN_USE != 0 {
            out[n] = (join(slot.hi.load(Ordering::Relaxed), lo), count);
            n += 1;
            slot.lo.store(0, Ordering::Release);
        }
    }
    n
}
