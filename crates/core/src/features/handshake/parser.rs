//! Parse a raw 802.11 data frame into an EAPOL-Key handshake message, if that is what it is.
//! Pure function over bytes (mirrors `scan/frame.rs::parse_beacon`): no allocation beyond the bounded
//! output, safe to call from the Wi-Fi sniffer callback.
use beewave_protocol::MAX_EAPOL_CAPTURE;

use super::types::EapolFrame;

const BASE_HEADER_LEN: usize = 24;
const QOS_EXTRA: usize = 2;
const LLC_SNAP: [u8; 6] = [0xAA, 0xAA, 0x03, 0x00, 0x00, 0x00];
const ETHERTYPE_EAPOL: [u8; 2] = [0x88, 0x8E];
const EAPOL_TYPE_KEY: u8 = 3;
/// EAPOL header (4) + the fixed EAPOL-Key body up to and including `key_data_length` (95).
const EAPOL_KEY_FIXED_LEN: usize = 4 + 95;

const ADDR1: core::ops::Range<usize> = 4..10;
const ADDR2: core::ops::Range<usize> = 10..16;

/// `None` if `frame` is not a well-formed EAPOL-Key frame of a recognized handshake message.
pub fn parse_eapol(frame: &[u8]) -> Option<EapolFrame> {
    if frame.len() < BASE_HEADER_LEN + 2 {
        return None;
    }
    let fc0 = frame[0] & 0xFC; // type + subtype, protocol-version bits masked off
    let is_data = fc0 == 0x08 || fc0 == 0x88; // Data, QoS Data
    if !is_data || frame[1] & 0x80 != 0 {
        return None; // not a plain data frame, or the Order bit (HT Control) is set: not handled
    }
    let to_ds = frame[1] & 0x01 != 0;
    let from_ds = frame[1] & 0x02 != 0;
    if to_ds && from_ds {
        return None; // WDS: four-address frame, not handled
    }
    let header_len = BASE_HEADER_LEN + if fc0 == 0x88 { QOS_EXTRA } else { 0 };
    let body = frame.get(header_len..)?;
    if body.len() < LLC_SNAP.len() + ETHERTYPE_EAPOL.len() {
        return None;
    }
    if body[..LLC_SNAP.len()] != LLC_SNAP || body[6..8] != ETHERTYPE_EAPOL {
        return None;
    }
    let eapol = &body[8..];
    if eapol.len() < EAPOL_KEY_FIXED_LEN || eapol[1] != EAPOL_TYPE_KEY {
        return None;
    }
    let key_info = u16::from_be_bytes([eapol[5], eapol[6]]);
    let message = classify(key_info)?;
    let replay_counter = u64::from_be_bytes(eapol[9..17].try_into().ok()?);

    let (bssid, client) = if from_ds {
        (frame[ADDR2].try_into().ok()?, frame[ADDR1].try_into().ok()?)
    } else if to_ds {
        (frame[ADDR1].try_into().ok()?, frame[ADDR2].try_into().ok()?)
    } else {
        // Neither ToDS nor FromDS (IBSS): no AP/STA distinction to make; address 3 is the best guess at BSSID.
        (
            frame.get(16..22)?.try_into().ok()?,
            frame[ADDR2].try_into().ok()?,
        )
    };

    let truncated = eapol.len() > MAX_EAPOL_CAPTURE;
    let eapol = heapless::Vec::from_slice(&eapol[..eapol.len().min(MAX_EAPOL_CAPTURE)]).ok()?;
    Some(EapolFrame {
        bssid,
        client,
        message,
        replay_counter,
        truncated,
        eapol,
    })
}

/// Message number from the Key Info bits (IEEE 802.11 Figure 12-32): bit 6 Install, bit 7 Key ACK,
/// bit 8 Key MIC, bit 9 Secure. `None` for anything that is not one of the four handshake messages
/// (for example a group-key frame).
fn classify(key_info: u16) -> Option<u8> {
    let pairwise = key_info & (1 << 3) != 0;
    if !pairwise {
        return None; // Group Key Handshake frames can match the same ACK/MIC/Secure pattern as messages 3/4
    }
    let ack = key_info & (1 << 7) != 0;
    let mic = key_info & (1 << 8) != 0;
    let secure = key_info & (1 << 9) != 0;
    match (ack, mic, secure) {
        (true, false, false) => Some(1),
        (false, true, false) => Some(2),
        (true, true, true) => Some(3),
        (false, true, true) => Some(4),
        _ => None,
    }
}
