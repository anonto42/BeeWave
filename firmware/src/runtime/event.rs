use beewave_protocol::DeviceCommand;

/// Control events the app task reacts to. (Sniffed frames travel on their own queue so a flood of
/// beacons can never delay or drop a command.)
pub enum Event {
    /// A command arrived from the client over BLE.
    Command(DeviceCommand),
}
