//! Parse a raw 802.11 management frame (beacon / probe response) into an `Observation`.
//! Pure function over bytes, so it is unit-tested on the host and safe to call from the
//! Wi-Fi sniffer callback (no allocation, no blocking).
use heapless::String;

use super::phy::{classify_phy, PhyInfo};
use super::security::{classify_security, parse_rsn_element, SecurityInfo, WPA1_OUI_TYPE};
use super::types::Observation;
use super::wps::parse_wps_element;

/// 802.11 management header is 24 bytes; a beacon body starts with 12 fixed bytes
/// (timestamp 8, interval 2, capabilities 2), then tagged information elements.
const MGMT_HEADER_LEN: usize = 24;
const BEACON_FIXED_LEN: usize = 12;
const IE_START: usize = MGMT_HEADER_LEN + BEACON_FIXED_LEN;
/// BSSID (address 3) position inside the management header.
const BSSID_RANGE: core::ops::Range<usize> = 16..22;

/// Frame-control byte 0 with the 2 protocol-version bits masked off: type + subtype.
const SUBTYPE_BEACON: u8 = 0x80;
const SUBTYPE_PROBE_RESPONSE: u8 = 0x50;

const IE_SSID: u8 = 0;
const IE_SUPPORTED_RATES: u8 = 1;
const IE_DS_PARAMS: u8 = 3;
const IE_ERP: u8 = 42;
const IE_HT_CAPABILITIES: u8 = 45;
const IE_RSN: u8 = 48;
const IE_EXT_SUPPORTED_RATES: u8 = 50;
const IE_VENDOR_SPECIFIC: u8 = 221;
const IE_EXTENSION: u8 = 255;
const EXT_ID_HE_CAPABILITIES: u8 = 35;

/// `None` if the frame is not a well-formed beacon/probe response.
/// `rx_channel` is the channel the radio was on; the DS Parameter Set element overrides it when present.
pub fn parse_beacon(frame: &[u8], rssi: i8, rx_channel: u8) -> Option<Observation> {
    if frame.len() < IE_START {
        return None;
    }
    let kind = frame[0] & 0xFC;
    if kind != SUBTYPE_BEACON && kind != SUBTYPE_PROBE_RESPONSE {
        return None;
    }
    let bssid: [u8; 6] = frame[BSSID_RANGE].try_into().ok()?;
    let tsf = u64::from_le_bytes([
        frame[24], frame[25], frame[26], frame[27], frame[28], frame[29], frame[30], frame[31],
    ]);

    let caps = u16::from_le_bytes([frame[34], frame[35]]);
    let privacy = (caps & 0x0010) != 0;

    let mut ssid = String::new();
    let mut channel = rx_channel;
    let mut wps = None;
    let mut rsn = None;
    let mut has_wpa1 = false;
    let mut has_he = false;
    let mut has_ht = false;
    let mut has_erp = false;
    let mut has_g_rates = false;
    let mut has_b_rates = false;
    let mut i = IE_START;
    while i + 2 <= frame.len() {
        let id = frame[i];
        let len = frame[i + 1] as usize;
        let end = i + 2 + len;
        if end > frame.len() {
            break; // truncated element: use what we have
        }
        let body = &frame[i + 2..end];
        match id {
            IE_SSID => ssid = parse_ssid(body),
            IE_SUPPORTED_RATES | IE_EXT_SUPPORTED_RATES => {
                for &r in body {
                    let rate_val = r & 0x7F;
                    if rate_val == 12 || rate_val == 18 || rate_val >= 24 {
                        has_g_rates = true;
                    } else if rate_val == 2 || rate_val == 4 || rate_val == 11 || rate_val == 22 {
                        has_b_rates = true;
                    }
                }
            }
            IE_DS_PARAMS if len == 1 => channel = body[0],
            IE_ERP => has_erp = true,
            IE_HT_CAPABILITIES => has_ht = true,
            IE_RSN if rsn.is_none() => rsn = Some(parse_rsn_element(body)),
            IE_EXTENSION if len >= 1 && body[0] == EXT_ID_HE_CAPABILITIES => {
                has_he = true;
            }
            IE_VENDOR_SPECIFIC => {
                if wps.is_none() {
                    if let Some(info) = parse_wps_element(bssid, body) {
                        wps = Some(info);
                    }
                }
                if !has_wpa1 && body.len() >= 4 && &body[0..4] == &WPA1_OUI_TYPE {
                    has_wpa1 = true;
                }
            }
            _ => {}
        }
        i = end;
    }

    let (protocol, pmf) = classify_security(privacy, rsn, has_wpa1);
    let security = Some(SecurityInfo {
        bssid,
        protocol,
        pmf,
    });
    let standard = classify_phy(has_he, has_ht, has_erp, has_g_rates, has_b_rates);
    let phy = Some(PhyInfo { bssid, standard });

    Some(Observation {
        bssid,
        channel,
        rssi,
        ssid,
        wps,
        security,
        phy,
        tsf,
    })
}

/// Hidden networks send an empty or all-zero SSID; non-UTF-8 or oversize SSIDs are dropped to "".
fn parse_ssid(body: &[u8]) -> String<32> {
    if body.len() > 32 || body.iter().all(|&b| b == 0) {
        return String::new();
    }
    core::str::from_utf8(body)
        .ok()
        .and_then(|s| String::try_from(s).ok())
        .unwrap_or_default()
}
