use std::sync::atomic::Ordering;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::sys::{self, esp, wifi_promiscuous_filter_t, EspError};
use esp_idf_svc::wifi::{ClientConfiguration, Configuration, EspWifi};
use walpad_core::features::radio::{AUTO_CHANNEL, MAX_CHANNEL};
use walpad_core::features::scan::Observation;
use walpad_core::ports::{RadioDriver, RadioError};

use super::callback;
use super::hopper;
use super::shared::Shared;
use crate::runtime::spawn_worker;

const HOPPER_STACK: usize = 3 * 1024;

pub struct WifiRadio {
    /// Kept alive so the Wi-Fi driver stays started (never connects to any network).
    _wifi: EspWifi<'static>,
    shared: Arc<Shared>,
}

impl WifiRadio {
    /// Start the Wi-Fi driver in station mode (without connecting) and arm the sniffer callback.
    pub fn new(modem: Modem, sysloop: EspSystemEventLoop, frames: SyncSender<Observation>) -> Result<Self, EspError> {
        let mut wifi = EspWifi::new(modem, sysloop, None)?;
        wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;
        wifi.start()?;

        callback::install(frames);
        let filter = wifi_promiscuous_filter_t { filter_mask: sys::WIFI_PROMIS_FILTER_MASK_MGMT };
        // SAFETY: Wi-Fi is started; `filter` is copied by the driver; `on_packet` has the required C signature.
        unsafe {
            esp!(sys::esp_wifi_set_promiscuous_filter(&filter))?;
            esp!(sys::esp_wifi_set_promiscuous_rx_cb(Some(callback::on_packet)))?;
        }
        hopper::set_channel(1)?;

        let shared = Arc::new(Shared::new());
        let for_hopper = shared.clone();
        spawn_worker("hopper", HOPPER_STACK, move || hopper::run(for_hopper));

        Ok(Self { _wifi: wifi, shared })
    }

    fn set_promiscuous(&self, on: bool) -> Result<(), RadioError> {
        // SAFETY: Wi-Fi is started (see `new`).
        esp!(unsafe { sys::esp_wifi_set_promiscuous(on) }).map_err(|_| RadioError::Driver)?;
        self.shared.sniffing.store(on, Ordering::Relaxed);
        Ok(())
    }
}

impl RadioDriver for WifiRadio {
    fn set_channel(&mut self, channel: u8) -> Result<(), RadioError> {
        if channel != AUTO_CHANNEL {
            if channel > MAX_CHANNEL {
                return Err(RadioError::InvalidChannel);
            }
            hopper::set_channel(channel).map_err(|_| RadioError::Driver)?;
            self.shared.current.store(channel, Ordering::Relaxed);
        }
        self.shared.pinned.store(channel, Ordering::Relaxed);
        Ok(())
    }

    fn start_sniffing(&mut self) -> Result<(), RadioError> {
        self.set_promiscuous(true)
    }

    fn stop_sniffing(&mut self) -> Result<(), RadioError> {
        self.set_promiscuous(false)
    }

    fn channel(&self) -> u8 {
        self.shared.current.load(Ordering::Relaxed)
    }
}
