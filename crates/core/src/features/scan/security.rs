use beewave_protocol::{WifiPmfMode, WifiSecurityProtocol};

pub const SECURITY_CAPACITY: usize = 32;

pub const RSN_OUI: [u8; 3] = [0x00, 0x0F, 0xAC];
pub const WPA1_OUI_TYPE: [u8; 4] = [0x00, 0x50, 0xF2, 0x01];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SecurityInfo {
    pub bssid: [u8; 6],
    pub protocol: WifiSecurityProtocol,
    pub pmf: WifiPmfMode,
}

/// Bounded table deduplicating security capability reports.
pub struct SecurityTracker {
    entries: heapless::Vec<SecurityInfo, SECURITY_CAPACITY>,
}

impl SecurityTracker {
    pub const fn new() -> Self {
        Self {
            entries: heapless::Vec::new(),
        }
    }

    /// Returns `true` if this security info should be emitted (new BSSID or security posture changed).
    pub fn observe(&mut self, info: &SecurityInfo) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.bssid == info.bssid) {
            if *entry == *info {
                false
            } else {
                *entry = *info;
                true
            }
        } else {
            if self.entries.is_full() {
                self.entries.swap_remove(0);
            }
            let _ = self.entries.push(*info);
            true
        }
    }
}

impl Default for SecurityTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Parses an 802.11 RSN element body (Tag 48 / 0x30).
/// Returns `(has_dot1x, has_psk, has_sae, has_owe, pmf)`.
pub fn parse_rsn_element(body: &[u8]) -> (bool, bool, bool, bool, WifiPmfMode) {
    if body.len() < 2 {
        return (false, false, false, false, WifiPmfMode::None);
    }
    // Version: 2 bytes LE (offset 0..2)
    let mut offset = 2;
    // Group cipher suite: 4 bytes (offset 2..6)
    if body.len() < offset + 4 {
        return (false, false, false, false, WifiPmfMode::None);
    }
    offset += 4;
    // Pairwise cipher suite count: 2 bytes LE
    if body.len() < offset + 2 {
        return (false, false, false, false, WifiPmfMode::None);
    }
    let pairwise_count = u16::from_le_bytes([body[offset], body[offset + 1]]) as usize;
    offset += 2;
    let pairwise_bytes = pairwise_count.saturating_mul(4);
    if body.len() < offset + pairwise_bytes {
        return (false, false, false, false, WifiPmfMode::None);
    }
    offset += pairwise_bytes;
    // AKM suite count: 2 bytes LE
    if body.len() < offset + 2 {
        return (false, false, false, false, WifiPmfMode::None);
    }
    let akm_count = u16::from_le_bytes([body[offset], body[offset + 1]]) as usize;
    offset += 2;
    let akm_bytes = akm_count.saturating_mul(4);
    if body.len() < offset + akm_bytes {
        return (false, false, false, false, WifiPmfMode::None);
    }

    let mut has_dot1x = false;
    let mut has_psk = false;
    let mut has_sae = false;
    let mut has_owe = false;

    for k in 0..akm_count {
        let akm = &body[offset + k * 4..offset + (k + 1) * 4];
        if akm[0..3] == RSN_OUI {
            match akm[3] {
                1 | 3 => has_dot1x = true,
                2 | 4 | 6 => has_psk = true,
                8 | 9 => has_sae = true,
                18 => has_owe = true,
                _ => {}
            }
        }
    }
    offset += akm_bytes;

    let mut pmf = WifiPmfMode::None;
    // RSN Capabilities: 2 bytes LE
    if body.len() >= offset + 2 {
        let caps = u16::from_le_bytes([body[offset], body[offset + 1]]);
        let mfpc = (caps & 0x0040) != 0; // bit 6
        let mfpr = (caps & 0x0080) != 0; // bit 7
        if mfpr {
            pmf = WifiPmfMode::Required;
        } else if mfpc {
            pmf = WifiPmfMode::Capable;
        }
    }

    (has_dot1x, has_psk, has_sae, has_owe, pmf)
}

/// Classifies security protocol and PMF mode from parsed beacon components.
pub fn classify_security(
    privacy: bool,
    rsn: Option<(bool, bool, bool, bool, WifiPmfMode)>,
    has_wpa1: bool,
) -> (WifiSecurityProtocol, WifiPmfMode) {
    if let Some((has_dot1x, has_psk, has_sae, has_owe, mut pmf)) = rsn {
        let proto = if has_sae && has_psk {
            WifiSecurityProtocol::Wpa2Wpa3
        } else if has_sae {
            WifiSecurityProtocol::Wpa3
        } else if has_owe {
            WifiSecurityProtocol::Owe
        } else if has_dot1x {
            WifiSecurityProtocol::Enterprise
        } else {
            WifiSecurityProtocol::Wpa2
        };

        // WPA3 and OWE require PMF by spec
        if (proto == WifiSecurityProtocol::Wpa3 || proto == WifiSecurityProtocol::Owe)
            && pmf == WifiPmfMode::None
        {
            pmf = WifiPmfMode::Required;
        } else if proto == WifiSecurityProtocol::Wpa2Wpa3 && pmf == WifiPmfMode::None {
            pmf = WifiPmfMode::Capable;
        }

        (proto, pmf)
    } else if has_wpa1 {
        (WifiSecurityProtocol::Wpa, WifiPmfMode::None)
    } else if privacy {
        (WifiSecurityProtocol::Wep, WifiPmfMode::None)
    } else {
        (WifiSecurityProtocol::Open, WifiPmfMode::None)
    }
}
