use beewave_protocol::{HopOrder, RejectReason};

use super::*;
use crate::ports::{Settings, SettingsError, SettingsStore};

/// A store that counts what happens to it, so tests can prove a rejected save never reached it.
#[derive(Default)]
struct FakeStore {
    saved: Option<Settings>,
    revision: u32,
    key: Option<[u8; 32]>,
    saves: u32,
    erases: u32,
    fail: bool,
}

impl SettingsStore for FakeStore {
    fn load(&mut self) -> Settings {
        self.saved.clone().unwrap_or_default()
    }
    fn save(&mut self, s: &Settings) -> Result<(), SettingsError> {
        if self.fail {
            return Err(SettingsError::Storage);
        }
        self.saved = Some(s.clone());
        self.revision += 1;
        self.saves += 1;
        Ok(())
    }
    fn save_revision(&mut self) -> u32 {
        self.revision
    }
    fn load_key(&mut self) -> Option<[u8; 32]> {
        self.key
    }
    fn save_key(&mut self, k: &[u8; 32]) -> Result<(), SettingsError> {
        self.key = Some(*k);
        Ok(())
    }
    fn erase_all(&mut self) -> Result<(), SettingsError> {
        self.saved = None;
        self.key = None;
        self.revision = 0;
        self.erases += 1;
        Ok(())
    }
}

fn ok(s: &Settings) {
    assert_eq!(validate(s), Ok(()), "{s:?}");
}
fn rejects(s: &Settings, why: RejectReason) {
    assert_eq!(validate(s), Err(why), "{s:?}");
}
fn with(f: impl FnOnce(&mut Settings)) -> Settings {
    let mut s = Settings::default();
    f(&mut s);
    s
}

#[test]
fn the_defaults_are_valid_and_match_the_documented_values() {
    let d = Settings::default();
    ok(&d);
    assert_eq!(
        (
            d.scan.dedup_rssi_db,
            d.radio.hop_dwell_ms,
            d.telemetry.status_period_ms
        ),
        (6, 250, 1000)
    );
    assert_eq!(
        (d.alerts.deauth_threshold, d.alerts.deauth_window_ms),
        (10, 1000)
    );
    assert_eq!(
        (d.radio.default_channel, d.radio.hop_order),
        (0, HopOrder::Ascending)
    );
}

// ---- every range boundary ---------------------------------------------------------------------

#[test]
fn scan_interval_is_100_to_10000() {
    // the window rule would hide the upper end, so widen the window first
    let at = |v: u32| {
        with(|s| {
            s.scan.scan_interval_ms = v;
            s.alerts.deauth_window_ms = 5000;
        })
    };
    rejects(&at(99), RejectReason::ScanIntervalRange);
    ok(&at(100));
    ok(&at(2500));
    rejects(&at(10_000), RejectReason::WindowTooShort); // in range itself, but 2 x 10000 > any window
    rejects(&at(10_001), RejectReason::ScanIntervalRange);
    rejects(&at(0), RejectReason::ScanIntervalRange);
}

#[test]
fn dedup_is_1_to_30_db() {
    let at = |v: u8| with(|s| s.scan.dedup_rssi_db = v);
    rejects(&at(0), RejectReason::DedupRssiRange);
    ok(&at(1));
    ok(&at(30));
    rejects(&at(31), RejectReason::DedupRssiRange);
    rejects(&at(255), RejectReason::DedupRssiRange);
}

#[test]
fn the_channel_filter_may_only_use_bits_for_channels_1_to_13() {
    let at = |v: u16| with(|s| s.scan.channel_filter_mask = v);
    ok(&at(0)); // zero = all channels
    ok(&at(0b1_1111_1111_1111));
    ok(&at(1));
    rejects(&at(1 << 13), RejectReason::ChannelMaskRange); // "channel 14"
    rejects(&at(u16::MAX), RejectReason::ChannelMaskRange);
}

#[test]
fn the_default_channel_is_auto_or_1_to_13() {
    let at = |v: u8| with(|s| s.radio.default_channel = v);
    ok(&at(0));
    ok(&at(1));
    ok(&at(13));
    rejects(&at(14), RejectReason::DefaultChannelRange);
    rejects(&at(255), RejectReason::DefaultChannelRange);
}

#[test]
fn hop_dwell_is_100_to_2000_ms() {
    let at = |v: u16| with(|s| s.radio.hop_dwell_ms = v);
    rejects(&at(99), RejectReason::HopDwellRange);
    ok(&at(100));
    ok(&at(2000));
    rejects(&at(2001), RejectReason::HopDwellRange);
    rejects(&at(0), RejectReason::HopDwellRange);
}

#[test]
fn status_period_is_250_to_5000_ms() {
    let at = |v: u32| with(|s| s.telemetry.status_period_ms = v);
    rejects(&at(249), RejectReason::StatusPeriodRange);
    ok(&at(250));
    ok(&at(5000));
    rejects(&at(5001), RejectReason::StatusPeriodRange);
    rejects(&at(0), RejectReason::StatusPeriodRange);
}

#[test]
fn deauth_threshold_is_1_to_100() {
    let at = |v: u8| with(|s| s.alerts.deauth_threshold = v);
    rejects(&at(0), RejectReason::DeauthThresholdRange);
    ok(&at(1));
    ok(&at(100));
    rejects(&at(101), RejectReason::DeauthThresholdRange);
    rejects(&at(255), RejectReason::DeauthThresholdRange);
}

#[test]
fn deauth_window_is_200_to_5000_ms() {
    let at = |v: u16| {
        with(|s| {
            s.alerts.deauth_window_ms = v;
            s.scan.scan_interval_ms = 100; // keep the cross-field rule out of the way
        })
    };
    rejects(&at(199), RejectReason::DeauthWindowRange);
    ok(&at(200));
    ok(&at(5000));
    rejects(&at(5001), RejectReason::DeauthWindowRange);
    rejects(&at(0), RejectReason::DeauthWindowRange);
}

// ---- cross-field rules ------------------------------------------------------------------------

#[test]
fn a_custom_hop_order_needs_at_least_one_channel() {
    let custom = |list: [u8; 13]| {
        with(|s| {
            s.radio.hop_order = HopOrder::Custom;
            s.radio.custom_hop_list = list;
        })
    };
    rejects(&custom([0; 13]), RejectReason::HopListEmpty);
    ok(&custom([6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]));
    ok(&custom([13, 1, 6, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0]));
    rejects(
        &custom([14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        RejectReason::HopListInvalid,
    );
    rejects(
        &custom([255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        RejectReason::HopListInvalid,
    );
    rejects(
        &custom([1, 6, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        RejectReason::HopListInvalid,
    );
    // the list is ignored (and may be empty) unless the order is Custom
    ok(&with(|s| s.radio.hop_order = HopOrder::Ascending1_6_11));
}

#[test]
fn the_deauth_window_must_span_two_scan_intervals() {
    let at = |interval: u32, window: u16| {
        with(|s| {
            s.scan.scan_interval_ms = interval;
            s.alerts.deauth_window_ms = window;
        })
    };
    ok(&at(500, 1000));
    rejects(&at(501, 1000), RejectReason::WindowTooShort);
    ok(&at(2500, 5000));
    rejects(&at(2501, 5000), RejectReason::WindowTooShort);
    ok(&at(100, 200));
    rejects(&at(101, 200), RejectReason::WindowTooShort);
}

#[test]
fn the_hop_order_resolves_to_the_channels_to_visit() {
    let order = |o: HopOrder, list: [u8; 13]| {
        let s = with(|s| {
            s.radio.hop_order = o;
            s.radio.custom_hop_list = list;
        });
        hop_order(&s.radio)
            .iter()
            .copied()
            .collect::<std::vec::Vec<u8>>()
    };
    assert_eq!(
        order(HopOrder::Ascending, [0; 13]),
        (1..=13).collect::<std::vec::Vec<u8>>()
    );
    assert_eq!(
        order(HopOrder::Ascending1_6_11, [0; 13]),
        [1, 6, 11, 2, 7, 12, 3, 8, 13, 4, 9, 5, 10]
    );
    assert_eq!(
        order(HopOrder::Custom, [11, 0, 1, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0]),
        [11, 1, 6]
    );
}

// ---- the service: persistence, rejection, revision --------------------------------------------

#[test]
fn a_rejected_save_never_reaches_the_store_or_the_live_settings() {
    let mut s = SettingsService::new(FakeStore::default());
    let bad = with(|v| v.scan.dedup_rssi_db = 99);
    assert_eq!(s.save_all(bad), Err(RejectReason::DedupRssiRange));
    assert_eq!(s.store.saves, 0, "NVS untouched");
    assert_eq!(s.revision(), 0);
    assert_eq!(*s.current(), Settings::default());
}

#[test]
fn a_good_save_persists_adopts_and_bumps_the_revision() {
    let mut s = SettingsService::new(FakeStore::default());
    let good = with(|v| {
        v.scan.dedup_rssi_db = 12;
        v.radio.hop_dwell_ms = 400;
    });
    assert_eq!(s.save_all(good.clone()), Ok(1));
    assert_eq!(*s.current(), good);
    assert_eq!(
        s.save_all(good.clone()),
        Ok(2),
        "every save counts, even an identical one"
    );
    assert_eq!(s.store.saved, Some(good));
}

#[test]
fn a_storage_failure_keeps_the_old_values_and_says_why() {
    let mut s = SettingsService::new(FakeStore::default());
    s.store.fail = true;
    assert_eq!(
        s.save_all(with(|v| v.scan.dedup_rssi_db = 12)),
        Err(RejectReason::Storage)
    );
    assert_eq!(*s.current(), Settings::default());
    assert_eq!(s.revision(), 0);
}

#[test]
fn stored_settings_load_at_start_but_an_invalid_stored_set_is_not_trusted() {
    let stored = with(|v| v.scan.dedup_rssi_db = 12);
    let s = SettingsService::new(FakeStore {
        saved: Some(stored.clone()),
        ..Default::default()
    });
    assert_eq!(*s.current(), stored);

    let corrupt = with(|v| v.alerts.deauth_threshold = 0);
    let s = SettingsService::new(FakeStore {
        saved: Some(corrupt),
        ..Default::default()
    });
    assert_eq!(*s.current(), Settings::default());
}

#[test]
fn the_legacy_pair_still_works_through_the_new_model() {
    let mut s = SettingsService::new(FakeStore::default());
    assert_eq!(s.set_legacy(6, true), Ok(1));
    assert_eq!(
        (
            s.current().radio.default_channel,
            s.current().scan.scan_at_boot
        ),
        (6, true)
    );
    assert_eq!(
        s.set_legacy(14, false),
        Err(RejectReason::DefaultChannelRange)
    );
    assert_eq!(s.current().radio.default_channel, 6);
}

#[test]
fn factory_reset_erases_everything_and_restores_defaults() {
    let mut s = SettingsService::new(FakeStore::default());
    s.save_all(with(|v| v.scan.dedup_rssi_db = 12)).unwrap();
    s.save_key(&[7; 32]).unwrap();
    assert!(s.factory_reset().is_ok());
    assert_eq!((s.store.erases, s.revision(), s.load_key()), (1, 0, None));
    assert_eq!(*s.current(), Settings::default());
}
