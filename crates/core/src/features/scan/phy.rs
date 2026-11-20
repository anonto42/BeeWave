//! 802.11 PHY generation / physical standard tracking and deduplication.
use beewave_protocol::WifiPhyStandard;

pub const PHY_CAPACITY: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhyInfo {
    pub bssid: [u8; 6],
    pub standard: WifiPhyStandard,
}

/// Bounded table deduplicating physical layer standard reports.
pub struct PhyTracker {
    entries: heapless::Vec<PhyInfo, PHY_CAPACITY>,
}

impl PhyTracker {
    pub const fn new() -> Self {
        Self {
            entries: heapless::Vec::new(),
        }
    }

    /// Returns `true` if this PHY info should be emitted (new BSSID or standard changed).
    pub fn observe(&mut self, info: &PhyInfo) -> bool {
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

impl Default for PhyTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Classifies physical standard from detected 802.11 capability flags and rates.
pub fn classify_phy(
    has_he: bool,
    has_ht: bool,
    has_erp: bool,
    has_g_rates: bool,
    has_b_rates: bool,
) -> WifiPhyStandard {
    if has_he {
        WifiPhyStandard::Ax
    } else if has_ht {
        WifiPhyStandard::N
    } else if has_erp || has_g_rates {
        WifiPhyStandard::G
    } else if has_b_rates {
        WifiPhyStandard::B
    } else {
        WifiPhyStandard::Unknown
    }
}
