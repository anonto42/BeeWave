//! Parse a raw 802.11 management frame into a `ProbeObservation`.
//! Pure function over bytes: zero allocation, safe to call from the sniffer callback.
use heapless::String;

use super::types::ProbeObservation;

/// 802.11 management header is 24 bytes (Frame Control 2, Duration 2, Addr1 6, Addr2 6, Addr3 6, Seq 2).
/// In Probe Requests, Information Elements start immediately at offset 24 (no fixed parameter block).
const MGMT_HEADER_LEN: usize = 24;

/// Frame-control byte 0 masked with 0xFC: type 00 (Management) + subtype 0100 (Probe Request) = 0x40.
const SUBTYPE_PROBE_REQUEST: u8 = 0x40;

/// Transmitter Address (Source Address / Client MAC) in the 802.11 management header.
const CLIENT_ADDR_RANGE: core::ops::Range<usize> = 10..16;

const IE_SSID: u8 = 0;
const IE_DS_PARAMS: u8 = 3;

/// `None` if the frame is not a well-formed 802.11 Probe Request frame.
pub fn parse_probe_request(frame: &[u8], rssi: i8, rx_channel: u8) -> Option<ProbeObservation> {
    if frame.len() < MGMT_HEADER_LEN {
        return None;
    }
    let kind = frame[0] & 0xFC;
    if kind != SUBTYPE_PROBE_REQUEST {
        return None;
    }
    let client: [u8; 6] = frame[CLIENT_ADDR_RANGE].try_into().ok()?;

    let mut ssid = String::new();
    let mut channel = rx_channel;
    let mut i = MGMT_HEADER_LEN;

    while i + 2 <= frame.len() {
        let id = frame[i];
        let len = frame[i + 1] as usize;
        let end = i + 2 + len;
        if end > frame.len() {
            break; // truncated element: use what was parsed
        }
        let body = &frame[i + 2..end];
        match id {
            IE_SSID => ssid = parse_ssid(body),
            IE_DS_PARAMS if len == 1 => channel = body[0],
            _ => {}
        }
        i = end;
    }

    Some(ProbeObservation {
        client,
        channel,
        rssi,
        ssid,
    })
}

fn parse_ssid(body: &[u8]) -> String<32> {
    if body.is_empty() || body.len() > 32 || body.iter().all(|&b| b == 0) {
        return String::new();
    }
    core::str::from_utf8(body)
        .ok()
        .and_then(|s| String::try_from(s).ok())
        .unwrap_or_default()
}
