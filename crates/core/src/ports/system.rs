/// Device-level facts and actions.
pub trait System {
    fn free_heap(&self) -> u32;
    fn cpu_load(&self) -> u8;
    fn restart(&mut self);
    /// Fill `buf` with cryptographically secure random bytes (hardware RNG on the device).
    fn fill_random(&mut self, buf: &mut [u8]);
}
