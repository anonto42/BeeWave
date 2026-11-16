//! The promiscuous-mode callback. It runs in the Wi-Fi driver task, so it must be fast:
//! no logging, no allocation, no blocking. Parse, then `try_send` and move on.
use core::ffi::c_void;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::OnceLock;

use beewave_core::features::scan::{parse_beacon, Observation};
use esp_idf_svc::sys::{wifi_promiscuous_pkt_t, wifi_promiscuous_pkt_type_t};

/// Longest frame we look at (beacons with many elements can be a few hundred bytes).
const MAX_FRAME: usize = 512;

/// The callback is a plain C function pointer, so its output queue lives in a static.
static FRAMES: OnceLock<SyncSender<Observation>> = OnceLock::new();

/// Frames dropped because the queue was full (visible in logs/debug later).
pub static DROPPED: AtomicU32 = AtomicU32::new(0);

pub fn install(tx: SyncSender<Observation>) {
    let _ = FRAMES.set(tx); // first call wins; we only ever install once
}

/// # Safety
/// Called by the Wi-Fi driver with a valid `wifi_promiscuous_pkt_t` in `buf` (we filter MGMT only).
pub unsafe extern "C" fn on_packet(buf: *mut c_void, _kind: wifi_promiscuous_pkt_type_t) {
    if buf.is_null() {
        return;
    }
    let pkt = &*(buf as *const wifi_promiscuous_pkt_t);
    let len = (pkt.rx_ctrl.sig_len() as usize).min(MAX_FRAME);
    let frame = core::slice::from_raw_parts(pkt.payload.as_ptr(), len);
    let rssi = pkt.rx_ctrl.rssi() as i8;
    let channel = pkt.rx_ctrl.channel() as u8;

    if let Some(obs) = parse_beacon(frame, rssi, channel) {
        if let Some(tx) = FRAMES.get() {
            if tx.try_send(obs).is_err() {
                DROPPED.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}
