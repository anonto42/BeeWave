//! Composition root: the only place that knows about concrete adapters.
//! Reading order: `runtime/` (threads + queues) -> `adapters/` (hardware) -> this file (wiring).
mod adapters;
mod runtime;

use std::time::Duration;

use adapters::ble_gatt::{self, BleSink};
use adapters::{esp_system::EspSystem, flash_ota::FlashOta, wifi_radio::WifiRadio};
use runtime::{channels, core_id, run_app, spawn_worker};
use walpad_core::app::App;

const BLE_STACK: usize = 6 * 1024;
const APP_STACK: usize = 8 * 1024;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Reaching here = self-test passed; confirm this OTA image so the bootloader won't roll back.
    unsafe { esp_idf_svc::sys::esp_ota_mark_app_valid_cancel_rollback() };

    let ch = channels();

    // BLE task: GATT server in, telemetry out.
    let events_for_ble = ch.events_tx.clone();
    let tele_rx = ch.tele_rx;
    spawn_worker("ble", BLE_STACK, move || {
        if let Err(e) = ble_gatt::run(events_for_ble, tele_rx) {
            log::error!("ble failed: {e:?}");
        }
    });

    // App task: owns the core `App`, wired to the real adapters.
    let app = App::new(WifiRadio::new(ch.events_tx.clone()), FlashOta::default(), EspSystem::new(), BleSink(ch.tele_tx));
    let events_rx = ch.events_rx;
    spawn_worker("app", APP_STACK, move || run_app(app, events_rx));

    log::info!("walpad up on core {}", core_id());
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}
