//! Ports: the only things the core knows about the outside world.
use walpad_protocol::TelemetryMessage;

/// Driven port: send telemetry to whoever is listening (BLE today). Lossy by contract.
pub trait TelemetrySink {
    /// Returns false if the message was dropped (queue full / no client).
    fn publish(&mut self, msg: TelemetryMessage) -> bool;
}

/// Driven port: the 2.4 GHz radio.
pub trait RadioDriver {
    fn set_channel(&mut self, channel: u8) -> Result<(), RadioError>;
    fn start_sniffing(&mut self) -> Result<(), RadioError>;
    fn stop_sniffing(&mut self) -> Result<(), RadioError>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RadioError {
    InvalidChannel,
    Driver,
}

/// Driven port: device-level facts and actions.
pub trait System {
    fn free_heap(&self) -> u32;
    fn cpu_load(&self) -> u8;
    fn restart(&mut self);
}

/// Driven port: inactive-slot firmware writer.
pub trait OtaWriter {
    fn begin(&mut self, size: u32) -> Result<(), OtaError>;
    fn write(&mut self, data: &[u8]) -> Result<(), OtaError>;
    /// Verify SHA-256 of everything written, then mark the slot bootable.
    fn finish(&mut self, expected_sha256: &[u8; 32]) -> Result<(), OtaError>;
    fn abort(&mut self);
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OtaError {
    Unsupported,
    TooLarge,
    BadOffset,
    BadState,
    HashMismatch,
    Flash,
}
