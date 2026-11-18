use heapless::String;

pub const WPS_OUI_TYPE: [u8; 4] = [0x00, 0x50, 0xF2, 0x04];
pub const WPS_CAPACITY: usize = 32;

// WPS TLV Attribute IDs
pub const ATTR_VERSION: u16 = 0x104A;
pub const ATTR_STATE: u16 = 0x1044;
pub const ATTR_AP_SETUP_LOCKED: u16 = 0x1057;
pub const ATTR_SELECTED_REGISTRAR: u16 = 0x1041;
pub const ATTR_DEVICE_NAME: u16 = 0x1011;
pub const ATTR_MANUFACTURER: u16 = 0x1021;
pub const ATTR_MODEL_NAME: u16 = 0x1023;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WpsInfo {
    pub bssid: [u8; 6],
    pub version: u8,
    pub state: u8,
    pub locked: bool,
    pub selected_registrar: bool,
    pub device_name: String<32>,
    pub manufacturer: String<32>,
    pub model_name: String<32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WpsFingerprint {
    pub bssid: [u8; 6],
    pub version: u8,
    pub state: u8,
    pub locked: bool,
    pub selected_registrar: bool,
}

/// Bounded table deduplicating WPS metadata reports.
pub struct WpsTracker {
    entries: heapless::Vec<WpsFingerprint, WPS_CAPACITY>,
}

impl WpsTracker {
    pub const fn new() -> Self {
        Self {
            entries: heapless::Vec::new(),
        }
    }

    /// Returns `true` if this WPS info should be emitted (new BSSID or status changed).
    pub fn observe(&mut self, info: &WpsInfo) -> bool {
        let fp = WpsFingerprint {
            bssid: info.bssid,
            version: info.version,
            state: info.state,
            locked: info.locked,
            selected_registrar: info.selected_registrar,
        };

        if let Some(entry) = self.entries.iter_mut().find(|e| e.bssid == info.bssid) {
            if *entry == fp {
                false
            } else {
                *entry = fp;
                true
            }
        } else {
            if self.entries.is_full() {
                self.entries.swap_remove(0);
            }
            let _ = self.entries.push(fp);
            true
        }
    }
}

impl Default for WpsTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Parses an 802.11 vendor-specific Information Element (ID 221) body for WPS attributes.
pub fn parse_wps_element(bssid: [u8; 6], body: &[u8]) -> Option<WpsInfo> {
    if body.len() < 4 || &body[0..4] != &WPS_OUI_TYPE {
        return None;
    }

    let mut version = 0x10; // Default 1.0
    let mut state = 0;
    let mut locked = false;
    let mut selected_registrar = false;
    let mut device_name = String::new();
    let mut manufacturer = String::new();
    let mut model_name = String::new();

    let mut i = 4;
    while i + 4 <= body.len() {
        let attr_type = u16::from_be_bytes([body[i], body[i + 1]]);
        let attr_len = u16::from_be_bytes([body[i + 2], body[i + 3]]) as usize;
        let end = i + 4 + attr_len;
        if end > body.len() {
            break;
        }
        let val = &body[i + 4..end];
        match attr_type {
            ATTR_VERSION if !val.is_empty() => version = val[0],
            ATTR_STATE if !val.is_empty() => state = val[0],
            ATTR_AP_SETUP_LOCKED if !val.is_empty() => locked = val[0] != 0,
            ATTR_SELECTED_REGISTRAR if !val.is_empty() => selected_registrar = val[0] != 0,
            ATTR_DEVICE_NAME => device_name = parse_string(val),
            ATTR_MANUFACTURER => manufacturer = parse_string(val),
            ATTR_MODEL_NAME => model_name = parse_string(val),
            _ => {}
        }
        i = end;
    }

    Some(WpsInfo {
        bssid,
        version,
        state,
        locked,
        selected_registrar,
        device_name,
        manufacturer,
        model_name,
    })
}

fn parse_string(val: &[u8]) -> String<32> {
    let mut s = String::new();
    if let Ok(text) = core::str::from_utf8(val) {
        for ch in text.chars().take(32) {
            let _ = s.push(ch);
        }
    }
    s
}
