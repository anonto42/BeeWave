use walpad_core::features::scan::Observation;
use walpad_protocol::DeviceCommand;

/// Everything the app task reacts to. Driving adapters send these; the app task receives them.
pub enum Event {
    /// A command arrived from the client over BLE.
    Command(DeviceCommand),
    /// The Wi-Fi sniffer saw a frame. Not constructed until the sniffer adapter is wired.
    #[allow(dead_code)]
    Frame(Observation),
}
