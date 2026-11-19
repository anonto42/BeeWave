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
    ble_observer,
    esp_system::{self, EspSystem},
    flash_ota::FlashOta,
    nvs_settings::NvsSettings,
    wifi_radio::{self, WifiRadio},
};
use beewave_core::app::App;
use runtime::{channels, core_id, run_app, spawn_worker, Hooks};

const BLE_STACK: usize = 6 * 1024;
/// Passive BLE observer thread (scan loop + classifier).
const BLE_OBSERVER_STACK: usize = 6 * 1024;
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

    // Passive BLE advertisement observer: idle until the owner switches the BLE monitor on.
    let ble_tx = ch.ble_tx;
    spawn_worker("ble_scan", BLE_OBSERVER_STACK, move || {
        ble_observer::run(ble_tx, ble_gatt::is_ready)
    });

    // App task: owns the core `App`, wired to the real adapters.
    let radio = WifiRadio::new(peripherals.modem, sysloop, ch.frames_tx)?;
    wifi_radio::install_handshake(ch.handshake_tx);
    wifi_radio::install_probes(ch.probes_tx);
    wifi_radio::install_assoc(ch.assoc_tx);
    let nvs_for_system = nvs.clone();
    let settings = NvsSettings::new(nvs)?;
    let keys = config::device_keys();
    let app = App::new(
        radio,
        FlashOta::default(),
        EspSystem::new(nvs_for_system, config::ota_fingerprint(&keys)),
        BleSink(ch.tele_tx),
        settings,
        keys,
    );
    let (events_rx, frames_rx, ble_rx, handshake_rx, probes_rx, assoc_rx) =
        (ch.events_rx, ch.frames_rx, ch.ble_rx, ch.handshake_rx, ch.probes_rx, ch.assoc_rx);
    spawn_worker("app", APP_STACK, move || {
        run_app(
            app,
            events_rx,
            frames_rx,
            ble_rx,
            handshake_rx,
            probes_rx,
            assoc_rx,
            Hooks {
                drain_deauth: wifi_radio::deauth::drain,
                drain_disassoc: wifi_radio::disassoc::drain,
                set_ble_observer: ble_observer::set_enabled,
                set_handshake_capture: wifi_radio::set_handshake_capture,
                set_log_level: esp_system::set_log_level,
            },
        )
    });

    log::info!("beewave up on core {}", core_id());
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}
