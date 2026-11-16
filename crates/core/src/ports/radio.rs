/// The 2.4 GHz radio.
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
