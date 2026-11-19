//! The promiscuous-mode callback. It runs in the Wi-Fi driver task, so it must be fast:
//! no logging, no allocation, no blocking. Parse, then `try_send` and move on.
use core::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::OnceLock;

use beewave_core::features::assoc::{parse_data_association, AssocObservation};
use beewave_core::features::handshake::{parse_eapol, EapolFrame};
use beewave_core::features::probe::{parse_probe_request, ProbeObservation};
use beewave_core::features::scan::{parse_beacon, Observation};

use super::{deauth, disassoc};
use esp_idf_svc::sys::{wifi_promiscuous_pkt_t, wifi_promiscuous_pkt_type_t};

/// Only `WIFI_PKT_DATA`, where EAPOL-Key frames live.
const PKT_DATA: wifi_promiscuous_pkt_type_t =
    esp_idf_svc::sys::wifi_promiscuous_pkt_type_t_WIFI_PKT_DATA;

/// First frame-control byte of a deauthentication frame: management type, subtype 12 (0xC0).
const DEAUTH_FC0: u8 = 0xC0;
const DISASSOC_FC0: u8 = 0xA0;
/// Offset of address 3 (the BSSID of a management frame) and the shortest frame that has it.
const ADDR3: usize = 16;
const MIN_MGMT_LEN: usize = 24;

/// Longest frame we look at (beacons with many elements can be a few hundred bytes).
const MAX_FRAME: usize = 512;

/// The callback is a plain C function pointer, so its output queue lives in a static.
static FRAMES: OnceLock<SyncSender<Observation>> = OnceLock::new();
static PROBES: OnceLock<SyncSender<ProbeObservation>> = OnceLock::new();
static ASSOC: OnceLock<SyncSender<AssocObservation>> = OnceLock::new();
static HANDSHAKE: OnceLock<SyncSender<EapolFrame>> = OnceLock::new();

/// Off by default: the owner's `SetHandshakeCapture` switch, mirrored here so data frames are rejected
/// in one cheap branch when the listener is off, instead of running the EAPOL parser on every one.
static HANDSHAKE_CAPTURE: AtomicBool = AtomicBool::new(false);

/// Frames dropped because the queue was full (visible in logs/debug later).
pub static DROPPED: AtomicU32 = AtomicU32::new(0);

pub fn install(tx: SyncSender<Observation>) {
    let _ = FRAMES.set(tx); // first call wins; we only ever install once
}

pub fn install_probes(tx: SyncSender<ProbeObservation>) {
    let _ = PROBES.set(tx);
}

pub fn install_assoc(tx: SyncSender<AssocObservation>) {
    let _ = ASSOC.set(tx);
}

pub fn install_handshake(tx: SyncSender<EapolFrame>) {
    let _ = HANDSHAKE.set(tx);
}

/// Mirrors the owner's handshake-capture switch (called from the app task, like `ble_observer::set_enabled`).
pub fn set_handshake_capture(on: bool) {
    HANDSHAKE_CAPTURE.store(on, Ordering::Relaxed);
}

/// # Safety
/// Called by the Wi-Fi driver with a valid `wifi_promiscuous_pkt_t` in `buf` (we filter MGMT only).
pub unsafe extern "C" fn on_packet(buf: *mut c_void, kind: wifi_promiscuous_pkt_type_t) {
    if buf.is_null() {
        return;
    }
    let pkt = &*(buf as *const wifi_promiscuous_pkt_t);
    let len = (pkt.rx_ctrl.sig_len() as usize).min(MAX_FRAME);
    let frame = core::slice::from_raw_parts(pkt.payload.as_ptr(), len);
    let rssi = pkt.rx_ctrl.rssi() as i8;
    let channel = pkt.rx_ctrl.channel() as u8;

    // Receive-only: count deauthentication frames per BSSID (nothing is ever transmitted).
    if len >= MIN_MGMT_LEN && frame[0] == DEAUTH_FC0 {
        if let Ok(bssid) = <[u8; 6]>::try_from(&frame[ADDR3..ADDR3 + 6]) {
            deauth::record(&bssid);
        }
        return;
    }

    // Receive-only: count disassociation frames per BSSID.
    if len >= MIN_MGMT_LEN && frame[0] == DISASSOC_FC0 {
        if let Ok(bssid) = <[u8; 6]>::try_from(&frame[ADDR3..ADDR3 + 6]) {
            disassoc::record(&bssid);
        }
        return;
    }

    if let Some(obs) = parse_beacon(frame, rssi, channel) {
        if let Some(tx) = FRAMES.get() {
            if tx.try_send(obs).is_err() {
                DROPPED.fetch_add(1, Ordering::Relaxed);
            }
        }
        return;
    }

    if let Some(probe) = parse_probe_request(frame, rssi, channel) {
        if let Some(tx) = PROBES.get() {
            if tx.try_send(probe).is_err() {
                DROPPED.fetch_add(1, Ordering::Relaxed);
            }
        }
        return;
    }

    // Receive-only: classify 802.11 Data frame associations and EAPOL-Key frames from a handshake
    // that is happening on its own.
    if kind == PKT_DATA {
        if let Some(assoc) = parse_data_association(frame, rssi, channel) {
            if let Some(tx) = ASSOC.get() {
                if tx.try_send(assoc).is_err() {
                    DROPPED.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        if HANDSHAKE_CAPTURE.load(Ordering::Relaxed) {
            if let Some(f) = parse_eapol(frame) {
                if let Some(tx) = HANDSHAKE.get() {
                    if tx.try_send(f).is_err() {
                        DROPPED.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }
    }
}
