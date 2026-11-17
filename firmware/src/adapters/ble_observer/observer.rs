use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::SyncSender;
use std::time::Duration;

use beewave_core::features::ble_monitor::classify;
use beewave_protocol::AdClass;
use esp32_nimble::{BLEDevice, BLEScan};
use esp_idf_svc::hal::task::block_on;

// ---- Tuning knobs (coexistence) ----------------------------------------------------------------
/// Listen for this long out of every `SCAN_INTERVAL_MS` (a 25 % duty cycle keeps the GATT link and the
/// Wi-Fi sniffer healthy; raise it only after checking `BLE LOST` on the bench).
const SCAN_WINDOW_MS: u16 = 40;
const SCAN_INTERVAL_MS: u16 = 160;
/// One scan run; the enable flag is re-checked between runs.
const RUN_MS: i32 = 4_000;
const IDLE_POLL: Duration = Duration::from_millis(500);

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Advertisements dropped because the queue to the app task was full.
pub static DROPPED: AtomicU32 = AtomicU32::new(0);

/// Switch the observer on or off (mirrors the owner's `SetMonitor` choice).
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

/// Runs forever on its own thread: scans in short runs while enabled, sleeps while not.
///
/// `stack_ready` must report when the GATT side has finished creating the NimBLE device: taking the
/// device concurrently with its creation panics (see `ble_gatt::is_ready`).
pub fn run(tx: SyncSender<AdClass>, stack_ready: fn() -> bool) {
    while !stack_ready() {
        std::thread::sleep(IDLE_POLL);
    }
    let device = BLEDevice::take();
    let mut scan = BLEScan::new();
    scan.active_scan(false) // passive: no scan requests are ever transmitted
        .filter_duplicates(false) // every advertisement counts towards the per-minute rate
        .interval(SCAN_INTERVAL_MS)
        .window(SCAN_WINDOW_MS);

    loop {
        if !ENABLED.load(Ordering::Relaxed) {
            std::thread::sleep(IDLE_POLL);
            continue;
        }
        let result = block_on(scan.start(device, RUN_MS, |_dev, data| {
            // NimBLE host task: classify (a handful of byte compares) and try_send, nothing else
            if tx.try_send(classify(data.payload())).is_err() {
                DROPPED.fetch_add(1, Ordering::Relaxed);
            }
            // stop this run as soon as the owner switches it off
            (!ENABLED.load(Ordering::Relaxed)).then_some(())
        }));
        if let Err(e) = result {
            log::warn!("ble observer: scan failed: {e:?}");
            std::thread::sleep(IDLE_POLL);
        }
    }
}
