use super::types::AssocObservation;

/// Minimum 802.11 MAC header length (24 bytes for 3-address frames).
const MIN_DATA_HEADER_LEN: usize = 24;
/// 802.11 Frame type mask (bits 2..3) and value for Data frames.
const TYPE_MASK: u8 = 0x0C;
const TYPE_DATA: u8 = 0x08;

/// 802.11 DS status bits in Frame Control byte 1.
const TO_DS_BIT: u8 = 0x01;
const FROM_DS_BIT: u8 = 0x02;

#[inline]
fn is_valid_unicast(mac: &[u8; 6]) -> bool {
    (mac[0] & 0x01) == 0 && *mac != [0; 6]
}

/// Zero-allocation parser for 802.11 Data and QoS Data frames.
///
/// Inspects the frame control type and the ToDS/FromDS flags to determine which address
/// is the Access Point BSSID and which is the Client Station MAC.
pub fn parse_data_association(frame: &[u8], rssi: i8, channel: u8) -> Option<AssocObservation> {
    if frame.len() < MIN_DATA_HEADER_LEN {
        return None;
    }

    let fc0 = frame[0];
    let fc1 = frame[1];

    // Must be Data type (0b10 in bits 2..3)
    if (fc0 & TYPE_MASK) != TYPE_DATA {
        return None;
    }

    let to_ds = (fc1 & TO_DS_BIT) != 0;
    let from_ds = (fc1 & FROM_DS_BIT) != 0;

    let mut addr1 = [0u8; 6];
    let mut addr2 = [0u8; 6];
    addr1.copy_from_slice(&frame[4..10]);
    addr2.copy_from_slice(&frame[10..16]);

    let (client, bssid) = match (to_ds, from_ds) {
        // Client -> AP: Addr1 is BSSID (RA), Addr2 is Client Station (TA/SA)
        (true, false) => (addr2, addr1),
        // AP -> Client: Addr1 is Client Station (RA/DA), Addr2 is BSSID (TA)
        (false, true) => (addr1, addr2),
        // Ad-hoc (IBSS 0,0) or WDS bridge (1,1): ignore
        _ => return None,
    };

    if is_valid_unicast(&client) && is_valid_unicast(&bssid) {
        Some(AssocObservation {
            bssid,
            client,
            channel,
            rssi,
        })
    } else {
        None
    }
}
