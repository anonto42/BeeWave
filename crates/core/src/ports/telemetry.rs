use walpad_protocol::TelemetryMessage;

/// Where telemetry goes (BLE notifications on the device). Lossy by contract.
pub trait TelemetrySink {
    /// Returns `false` if the message was dropped (queue full / nobody connected).
    fn publish(&mut self, msg: TelemetryMessage) -> bool;
}
