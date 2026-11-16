//! Composition root: the only place that knows about concrete adapters.
//! Reading order: `runtime/` (threads + queues) -> `adapters/` (hardware) -> this file (wiring).
mod adapters;
mod config;
mod runtime;

use std::time::Duration;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;

use adapters::ble_gatt::{self, BleSink};
use adapters::{
    esp_system::EspSystem, flash_ota::FlashOta, nvs_settings::NvsSettings, wifi_radio::WifiRadio,
};
use beewave_core::app::App;
use runtime::{channels, core_id, run_app, spawn_worker};

const BLE_STACK: usize = 6 * 1024;
const APP_STACK: usize = 16 * 1024;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Reaching here = self-test passed; confirm this OTA image so the bootloader won't roll back.
    unsafe { esp_idf_svc::sys::esp_ota_mark_app_valid_cancel_rollback() };

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
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
    let radio = WifiRadio::new(peripherals.modem, sysloop, ch.frames_tx)?;
    let settings = NvsSettings::new(nvs)?;
    let app = App::new(
        radio,
        FlashOta::default(),
        EspSystem::new(),
        BleSink(ch.tele_tx),
        settings,
        config::device_keys(),
    );
    let (events_rx, frames_rx) = (ch.events_rx, ch.frames_rx);
    spawn_worker("app", APP_STACK, move || run_app(app, events_rx, frames_rx));

    log::info!("beewave up on core {}", core_id());
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}
