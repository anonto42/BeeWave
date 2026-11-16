//! `System` port: heap, CPU load and restart from ESP-IDF.
use std::cell::Cell;
use std::ffi::CStr;

use esp_idf_svc::sys::{self, TaskStatus_t};
use walpad_core::ports::System;

/// ESP32 has two cores, each with its own idle task ("IDLE0" / "IDLE1").
const CORES: u64 = 2;

pub struct EspSystem {
    /// (idle runtime, total runtime) at the previous sample; load is computed over the interval.
    last: Cell<(u32, u32)>,
    last_load: Cell<u8>,
}

impl EspSystem {
    pub fn new() -> Self {
        Self { last: Cell::new(runtime_counters().unwrap_or((0, 0))), last_load: Cell::new(0) }
    }
}

/// `(sum of idle-task runtime, total runtime)` since boot, both in the same timer ticks.
fn runtime_counters() -> Option<(u32, u32)> {
    // SAFETY: we pass a buffer of `capacity` TaskStatus_t and FreeRTOS fills at most that many.
    unsafe {
        let capacity = sys::uxTaskGetNumberOfTasks() as usize + 4; // headroom for tasks spawned meanwhile
        let mut tasks: Vec<TaskStatus_t> = Vec::with_capacity(capacity);
        let mut total: u32 = 0;
        let got = sys::uxTaskGetSystemState(tasks.as_mut_ptr(), capacity as u32, &mut total) as usize;
        if got == 0 || total == 0 {
            return None;
        }
        tasks.set_len(got);
        let idle = tasks
            .iter()
            .filter(|t| CStr::from_ptr(t.pcTaskName).to_bytes().starts_with(b"IDLE"))
            .fold(0u32, |acc, t| acc.wrapping_add(t.ulRunTimeCounter));
        Some((idle, total))
    }
}

impl System for EspSystem {
    fn free_heap(&self) -> u32 {
        // SAFETY: plain query, no preconditions.
        unsafe { sys::esp_get_free_heap_size() }
    }

    fn cpu_load(&self) -> u8 {
        let Some((idle, total)) = runtime_counters() else { return self.last_load.get() };
        let (prev_idle, prev_total) = self.last.replace((idle, total));
        let d_total = total.wrapping_sub(prev_total) as u64;
        if d_total == 0 {
            return self.last_load.get();
        }
        let d_idle = idle.wrapping_sub(prev_idle) as u64;
        let idle_pct = (d_idle * 100 / (d_total * CORES)).min(100);
        let load = (100 - idle_pct) as u8;
        self.last_load.set(load);
        load
    }

    fn restart(&mut self) {
        // SAFETY: never returns; the chip reboots.
        unsafe { sys::esp_restart() }
    }
}
