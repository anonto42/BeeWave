//! Wi-Fi radio adapter. Driven: RadioDriver port. Driving: pushes sniffed frames as Events.
//! TODO: bring up Wi-Fi in NULL/STA mode, esp_wifi_set_promiscuous(true), MGMT-only filter,
//! callback -> preallocated ring -> this struct's pump thread -> Event::Frame.
use crate::runtime::Event;
use std::sync::mpsc::SyncSender;
use walpad_core::ports::{RadioDriver, RadioError};

pub struct WifiRadio {
    #[allow(dead_code)]
    events: SyncSender<Event>,
    sniffing: bool,
}

impl WifiRadio {
    pub fn new(events: SyncSender<Event>) -> Self { Self { events, sniffing: false } }
}

impl RadioDriver for WifiRadio {
    fn set_channel(&mut self, channel: u8) -> Result<(), RadioError> {
        log::info!("radio: channel {channel} (sniffer not wired yet)");
        Ok(())
    }
    fn start_sniffing(&mut self) -> Result<(), RadioError> { self.sniffing = true; Ok(()) }
    fn stop_sniffing(&mut self) -> Result<(), RadioError> { self.sniffing = false; Ok(()) }
}
