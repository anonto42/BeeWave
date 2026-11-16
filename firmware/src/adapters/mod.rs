//! Adapters: all ESP-IDF / hardware specifics live here, behind core ports.
//! Driving (outside -> core): ble_gatt (commands), wifi_radio (frames).
//! Driven  (core -> outside): ble_gatt (telemetry sink), wifi_radio, flash_ota, esp_system.
pub mod ble_gatt;
pub mod esp_system;
pub mod flash_ota;
pub mod wifi_radio;
