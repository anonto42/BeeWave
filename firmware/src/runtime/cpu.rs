use esp_idf_svc::hal::cpu;

/// Which core the calling thread is running on right now (0 or 1).
pub fn core_id() -> u32 {
    cpu::core() as u32
}
