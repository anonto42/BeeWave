//! Adapters: all ESP-IDF / hardware specifics live here, behind core ports.
//! Driving (outside -> core): ble_gatt (commands), wifi_radio (frames, deauth counts), ble_observer (passive
//! BLE advertisements).
//! Driven  (core -> outside): ble_gatt (telemetry sink), wifi_radio, flash_ota, esp_system.
pub mod ble_gatt;
pub mod ble_observer;
pub mod esp_system;
pub mod flash_ota;
pub mod nvs_settings;
pub mod wifi_radio;
