use beewave_protocol::{HopOrder, RadioSettings, RejectReason, HOP_LIST_LEN};

use crate::ports::{Settings, AUTO_CHANNEL, MAX_CHANNEL};

pub const SCAN_INTERVAL_MS: core::ops::RangeInclusive<u32> = 100..=10_000;
pub const DEDUP_RSSI_DB: core::ops::RangeInclusive<u8> = 1..=30;
pub const HOP_DWELL_MS: core::ops::RangeInclusive<u16> = 100..=2_000;
pub const STATUS_PERIOD_MS: core::ops::RangeInclusive<u32> = 250..=5_000;
pub const DEAUTH_THRESHOLD: core::ops::RangeInclusive<u8> = 1..=100;
pub const DEAUTH_WINDOW_MS: core::ops::RangeInclusive<u16> = 200..=5_000;
/// Bits 0-12 (channels 1-13) are the only ones that may be set in the channel filter.
pub const CHANNEL_MASK_BITS: u16 = (1 << MAX_CHANNEL) - 1;

/// Check every field and the rules that tie fields together. Reports the first problem, naming the field.
pub fn validate(s: &Settings) -> Result<(), RejectReason> {
    if !SCAN_INTERVAL_MS.contains(&s.scan.scan_interval_ms) {
        return Err(RejectReason::ScanIntervalRange);
    }
    if !DEDUP_RSSI_DB.contains(&s.scan.dedup_rssi_db) {
        return Err(RejectReason::DedupRssiRange);
    }
    if s.scan.channel_filter_mask & !CHANNEL_MASK_BITS != 0 {
        return Err(RejectReason::ChannelMaskRange);
    }
    let ch = s.radio.default_channel;
    if ch != AUTO_CHANNEL && !(1..=MAX_CHANNEL).contains(&ch) {
        return Err(RejectReason::DefaultChannelRange);
    }
    if !HOP_DWELL_MS.contains(&s.radio.hop_dwell_ms) {
        return Err(RejectReason::HopDwellRange);
    }
    if s.radio.hop_order == HopOrder::Custom {
        validate_hop_list(&s.radio.custom_hop_list)?;
    }
    if !STATUS_PERIOD_MS.contains(&s.telemetry.status_period_ms) {
        return Err(RejectReason::StatusPeriodRange);
    }
    if !DEAUTH_THRESHOLD.contains(&s.alerts.deauth_threshold) {
        return Err(RejectReason::DeauthThresholdRange);
    }
    if !DEAUTH_WINDOW_MS.contains(&s.alerts.deauth_window_ms) {
        return Err(RejectReason::DeauthWindowRange);
    }
    // the counting window must span at least two scan intervals, or one slow cycle could hide a burst
    if u32::from(s.alerts.deauth_window_ms) < 2 * s.scan.scan_interval_ms {
        return Err(RejectReason::WindowTooShort);
    }
    Ok(())
}

/// A custom list holds channels 1-13 once each, padded with 0, and at least one channel.
fn validate_hop_list(list: &[u8; HOP_LIST_LEN]) -> Result<(), RejectReason> {
    let mut seen = 0u16;
    let mut any = false;
    for &c in list.iter().filter(|&&c| c != 0) {
        if !(1..=MAX_CHANNEL).contains(&c) || seen & (1 << c) != 0 {
            return Err(RejectReason::HopListInvalid);
        }
        seen |= 1 << c;
        any = true;
    }
    any.then_some(()).ok_or(RejectReason::HopListEmpty)
}

/// The channels to visit, in order, for these radio settings (at most 13).
pub fn hop_order(r: &RadioSettings) -> heapless::Vec<u8, HOP_LIST_LEN> {
    let mut out = heapless::Vec::new();
    match r.hop_order {
        HopOrder::Ascending => (1..=MAX_CHANNEL).for_each(|c| {
            let _ = out.push(c);
        }),
        HopOrder::Ascending1_6_11 => [1, 6, 11, 2, 7, 12, 3, 8, 13, 4, 9, 5, 10]
            .into_iter()
            .for_each(|c| {
                let _ = out.push(c);
            }),
        HopOrder::Custom => r
            .custom_hop_list
            .iter()
            .filter(|&&c| c != 0)
            .for_each(|&c| {
                let _ = out.push(c);
            }),
    }
    out
}
