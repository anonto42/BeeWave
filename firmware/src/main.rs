//! Composition root: the only place that knows about concrete adapters.
mod adapters;
mod runtime;

use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use adapters::{ble_gatt, esp_system::EspSystem, flash_ota::FlashOta, wifi_radio::WifiRadio};
use runtime::{channels, spawn_worker, Event};
use walpad_core::app::App;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Reaching here = self-test passed; confirm this OTA image so the bootloader won't roll back.
    unsafe { esp_idf_svc::sys::esp_ota_mark_app_valid_cancel_rollback() };

    let ch = channels();

    let events_for_ble = ch.events_tx.clone();
    let tele_rx = ch.tele_rx;
    spawn_worker("ble", 6 * 1024, move || {
        if let Err(e) = ble_gatt::run(events_for_ble, tele_rx) { log::error!("ble failed: {e:?}"); }
    });

    let mut app = App::new(
        WifiRadio::new(ch.events_tx.clone()),
        FlashOta,
        EspSystem,
        ble_gatt::BleSink(ch.tele_tx),
    );
    let events_rx = ch.events_rx;
    spawn_worker("app", 8 * 1024, move || loop {
        match events_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Event::Command(c)) => app.on_command(c),
            Ok(Event::Frame(o)) => app.on_observation(o),
            Err(RecvTimeoutError::Timeout) => app.on_tick(),
            Err(RecvTimeoutError::Disconnected) => return,
        }
    });

    log::info!("walpad up on core {}", runtime::core_id());
    loop { std::thread::sleep(Duration::from_secs(60)); }
}
