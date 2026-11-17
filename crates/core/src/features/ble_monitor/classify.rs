use beewave_protocol::AdClass;

const AD_SERVICE_DATA_16: u8 = 0x16;
const AD_MANUFACTURER: u8 = 0xFF;
const COMPANY_APPLE: u16 = 0x004C;
const FAST_PAIR_UUID: u16 = 0xFE2C;
/// Apple manufacturer data sub-types.
const APPLE_IBEACON: u8 = 0x02;
const APPLE_IBEACON_LEN: u8 = 0x15;
const APPLE_PROXIMITY_PAIRING: u8 = 0x07;

/// Class of a raw advertisement payload (a sequence of `[len][type][data]` structures). A truncated
/// or malformed structure ends the walk; whatever was recognised before it still counts.
pub fn classify(payload: &[u8]) -> AdClass {
    let mut rest = payload;
    while let Some((&len, tail)) = rest.split_first() {
        let len = usize::from(len);
        if len == 0 || len > tail.len() {
            break;
        }
        let (structure, next) = tail.split_at(len);
        if let Some((&ty, data)) = structure.split_first() {
            if let Some(class) = classify_structure(ty, data) {
                return class;
            }
        }
        rest = next;
    }
    AdClass::Generic
}

fn classify_structure(ty: u8, data: &[u8]) -> Option<AdClass> {
    match ty {
        AD_SERVICE_DATA_16 if data.len() >= 2 => {
            (u16::from_le_bytes([data[0], data[1]]) == FAST_PAIR_UUID).then_some(AdClass::FastPair)
        }
        AD_MANUFACTURER if data.len() >= 4 => {
            if u16::from_le_bytes([data[0], data[1]]) != COMPANY_APPLE {
                return None;
            }
            match (data[2], data[3]) {
                (APPLE_IBEACON, APPLE_IBEACON_LEN) => Some(AdClass::IBeacon),
                (APPLE_PROXIMITY_PAIRING, _) => Some(AdClass::Proximity),
                _ => None,
            }
        }
        _ => None,
    }
}
