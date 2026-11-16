/// Device-level facts and actions.
pub trait System {
    fn free_heap(&self) -> u32;
    fn cpu_load(&self) -> u8;
    fn restart(&mut self);
}
