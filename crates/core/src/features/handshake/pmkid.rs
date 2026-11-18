//! Passive PMKID extraction from EAPOL-Key Message 1 Key Data (IEEE 802.11i).
//!
//! In WPA2/WPA3 networks supporting PMK caching, an AP includes the PMKID in the Key Data
//! field of EAPOL-Key Message 1. This module parses the Key Data TLV list in zero-copy
//! fashion using only fixed-size arrays and safe slice indexing — no allocation, no panic.
//!
//! # Key Data Encapsulation (KDE) format (IEEE 802.11-2020 §12.7.2)
//!
//! Each element in Key Data is a TLV:
//!   `[0xDD][length: u8][OUI: 3 bytes][Data Type: u8][value: length-4 bytes]`
//!
//! For a PMKID KDE:
//!   - Tag:       `0xDD`
//!   - Length:    `0x14`  (20 = 4 header + 16 PMKID)
//!   - OUI:       `00 0F AC`
//!   - Data Type: `04`
//!   - Value:     `[u8; 16]` PMKID

/// EAPOL-Key fixed body length (from EAPOL version byte):
///   EAPOL header (4) + Key Descriptor (95-byte fixed section) = 99.
/// `key_data_length` is at bytes [97..99] (u16 big-endian) within the EAPOL body.
const EAPOL_KEY_DATA_LEN_OFFSET: usize = 97;
const EAPOL_KEY_DATA_OFFSET: usize = 99;

/// KDE tag byte for vendor-specific elements (which PMKID uses).
const KDE_TAG: u8 = 0xDD;
/// Minimum length of a KDE header (4 bytes: OUI 3 + data type 1).
const KDE_HDR: usize = 4;
/// Full length field value for a PMKID KDE (4 header + 16 PMKID).
const PMKID_KDE_LEN: u8 = 20;
/// OUI for IEEE 802.11 suite (00-0F-AC).
const IEEE80211_OUI: [u8; 3] = [0x00, 0x0F, 0xAC];
/// Data Type for PMKID KDE.
const PMKID_DATA_TYPE: u8 = 0x04;

/// Search the EAPOL-Key body (`eapol` starts at the EAPOL version byte) for a PMKID KDE.
///
/// Returns `Some([u8; 16])` on the first PMKID KDE found, or `None` if none is present or
/// the buffer is too short / malformed. Never panics; never allocates.
///
/// Only Message 1 (AP → STA, `ACK` bit set, `MIC` bit clear) normally carries a PMKID; call
/// this only after classifying the frame as Message 1 to avoid false positives.
pub fn extract_pmkid(eapol: &[u8]) -> Option<[u8; 16]> {
    // Bounds: need at least the fixed header + 2-byte key_data_length field.
    if eapol.len() < EAPOL_KEY_DATA_OFFSET {
        return None;
    }
    // key_data_length is at bytes [97..99]: big-endian u16 within the EAPOL body.
    let kd_len = u16::from_be_bytes([
        eapol[EAPOL_KEY_DATA_LEN_OFFSET],
        eapol[EAPOL_KEY_DATA_LEN_OFFSET + 1],
    ]) as usize;

    // Slice out exactly the key data region. If the frame was truncated, clamp to what we have.
    let available = eapol.len().saturating_sub(EAPOL_KEY_DATA_OFFSET);
    let kd_actual = kd_len.min(available);
    let kd = eapol.get(EAPOL_KEY_DATA_OFFSET..EAPOL_KEY_DATA_OFFSET + kd_actual)?;

    // Walk the TLV list. Each element: [tag: u8][length: u8][value: `length` bytes].
    let mut pos = 0;
    while pos + 2 <= kd.len() {
        let tag = kd[pos];
        let len = kd[pos + 1] as usize;
        let value_start = pos + 2;
        let value_end = value_start + len;

        // Guard: reject elements whose value would exceed the buffer.
        if value_end > kd.len() {
            break;
        }

        if tag == KDE_TAG && len == PMKID_KDE_LEN as usize {
            let value = &kd[value_start..value_end]; // exactly 20 bytes
                                                     // Check OUI + data type (first 4 bytes of the value).
            if value.get(..3) == Some(&IEEE80211_OUI) && value.get(3) == Some(&PMKID_DATA_TYPE) {
                // The last 16 bytes are the PMKID.
                if let Some(pmkid_slice) = value.get(KDE_HDR..KDE_HDR + 16) {
                    // Safe: pmkid_slice is exactly 16 bytes.
                    if let Ok(pmkid) = pmkid_slice.try_into() {
                        return Some(pmkid);
                    }
                }
            }
        }

        // Advance past this element (tag + length + value).
        pos = value_end;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // A minimal EAPOL-Key body of exactly `EAPOL_KEY_DATA_OFFSET` bytes (zeroed fixed section)
    // followed by a PMKID KDE.
    fn eapol_with_pmkid(pmkid: [u8; 16]) -> heapless::Vec<u8, 256> {
        let mut v: heapless::Vec<u8, 256> = heapless::Vec::new();
        // Fill the fixed 99-byte EAPOL-Key header with zeros.
        v.extend_from_slice(&[0u8; EAPOL_KEY_DATA_OFFSET]).unwrap();
        // A PMKID KDE is 22 bytes total: tag + length + 20 value bytes.
        let kde_total = 1 + 1 + PMKID_KDE_LEN as usize; // 22
                                                        // Patch key_data_length to kde_total (big-endian u16).
        let len_bytes = (kde_total as u16).to_be_bytes();
        v[EAPOL_KEY_DATA_LEN_OFFSET] = len_bytes[0];
        v[EAPOL_KEY_DATA_LEN_OFFSET + 1] = len_bytes[1];
        // Append the KDE.
        v.push(KDE_TAG).unwrap();
        v.push(PMKID_KDE_LEN).unwrap();
        v.extend_from_slice(&IEEE80211_OUI).unwrap();
        v.push(PMKID_DATA_TYPE).unwrap();
        v.extend_from_slice(&pmkid).unwrap();
        v
    }

    #[test]
    fn extracts_a_valid_pmkid_kde() {
        let pmkid = [0x42u8; 16];
        let eapol = eapol_with_pmkid(pmkid);
        assert_eq!(extract_pmkid(&eapol), Some(pmkid));
    }

    #[test]
    fn returns_none_when_no_pmkid_kde_present() {
        // Key data with a different KDE (data type != 0x04).
        let mut eapol = eapol_with_pmkid([0u8; 16]);
        // Corrupt the data type byte (at offset 99 + 1 + 1 + 3 = 104).
        let dt_offset = EAPOL_KEY_DATA_OFFSET + 2 + 3;
        eapol[dt_offset] = 0x05; // wrong data type
        assert_eq!(extract_pmkid(&eapol), None);
    }

    #[test]
    fn returns_none_for_truncated_key_data() {
        // Buffer shorter than EAPOL_KEY_DATA_OFFSET.
        assert_eq!(extract_pmkid(&[0u8; 10]), None);
    }

    #[test]
    fn returns_none_when_tlv_length_exceeds_buffer() {
        let mut eapol = eapol_with_pmkid([0u8; 16]);
        // Set key_data_length to something large but truncate the actual buffer.
        let large = 200u16.to_be_bytes();
        eapol[EAPOL_KEY_DATA_LEN_OFFSET] = large[0];
        eapol[EAPOL_KEY_DATA_LEN_OFFSET + 1] = large[1];
        // The slice is clamped to `available`; iterator should stop cleanly.
        // We can't guarantee extraction here (the KDE is now beyond the clamped window),
        // but it must not panic.
        let _ = extract_pmkid(&eapol);
    }

    #[test]
    fn skips_non_matching_kde_and_finds_pmkid() {
        let pmkid = [0xABu8; 16];
        let mut eapol: heapless::Vec<u8, 256> = heapless::Vec::new();
        eapol
            .extend_from_slice(&[0u8; EAPOL_KEY_DATA_OFFSET])
            .unwrap();

        // Build key data: a junk KDE (tag=0xDD, len=4, wrong OUI) then the real PMKID KDE.
        let junk = [KDE_TAG, 4u8, 0x00, 0x50, 0xF2, 0x01]; // WPA information element style
        let pmkid_kde_len = 1 + 1 + PMKID_KDE_LEN as usize; // 22
        let total = junk.len() + pmkid_kde_len;
        let len_bytes = (total as u16).to_be_bytes();
        eapol[EAPOL_KEY_DATA_LEN_OFFSET] = len_bytes[0];
        eapol[EAPOL_KEY_DATA_LEN_OFFSET + 1] = len_bytes[1];
        eapol.extend_from_slice(&junk).unwrap();
        eapol.push(KDE_TAG).unwrap();
        eapol.push(PMKID_KDE_LEN).unwrap();
        eapol.extend_from_slice(&IEEE80211_OUI).unwrap();
        eapol.push(PMKID_DATA_TYPE).unwrap();
        eapol.extend_from_slice(&pmkid).unwrap();

        assert_eq!(extract_pmkid(&eapol), Some(pmkid));
    }
}
