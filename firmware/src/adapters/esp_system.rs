use walpad_core::ports::System;

pub struct EspSystem;

impl System for EspSystem {
    fn free_heap(&self) -> u32 { unsafe { esp_idf_svc::sys::esp_get_free_heap_size() } }
    fn cpu_load(&self) -> u8 { 0 } // TODO: FreeRTOS run-time stats
    fn restart(&mut self) { unsafe { esp_idf_svc::sys::esp_restart() } }
}
