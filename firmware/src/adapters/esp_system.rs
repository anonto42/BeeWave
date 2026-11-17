//! `System` port: heap, CPU load and restart from ESP-IDF.
use std::cell::Cell;
use std::ffi::CStr;

use beewave_core::ports::{BuildFacts, PartitionInfo, System};
use beewave_protocol::LogLevel;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
use esp_idf_svc::sys::{self, TaskStatus_t};

/// ESP32 has two cores, each with its own idle task ("IDLE0" / "IDLE1").
const CORES: u64 = 2;

/// NVS entries are 32 bytes each.
const NVS_ENTRY_BYTES: u32 = 32;
/// Reserved NVS namespace for the self-test scratch key (never holds anything the app reads).
const SELFTEST_NAMESPACE: &str = "selftest";

pub struct EspSystem {
    /// (idle runtime, total runtime) at the previous sample; load is computed over the interval.
    last: Cell<(u32, u32)>,
    last_load: Cell<u8>,
    nvs: EspDefaultNvsPartition,
    /// First 16 bytes of the SHA-256 of the embedded OTA public key (zeros when there is none).
    ota_key_fp: [u8; 16],
}

impl EspSystem {
    pub fn new(nvs: EspDefaultNvsPartition, ota_key_fp: [u8; 16]) -> Self {
        Self {
            last: Cell::new(runtime_counters().unwrap_or((0, 0))),
            last_load: Cell::new(0),
            nvs,
            ota_key_fp,
        }
    }
}

/// The partition of the given OTA slot, if the table has it.
fn ota_partition(subtype: sys::esp_partition_subtype_t) -> Option<&'static sys::esp_partition_t> {
    // SAFETY: the returned pointer is into the partition table kept by ESP-IDF for the life of the program.
    unsafe {
        sys::esp_partition_find_first(
            sys::esp_partition_type_t_ESP_PARTITION_TYPE_APP,
            subtype,
            core::ptr::null(),
        )
        .as_ref()
    }
}

/// Which OTA slot the running code is in.
fn running_slot() -> Option<sys::esp_partition_subtype_t> {
    // SAFETY: plain query; null means "unknown".
    let running = unsafe { sys::esp_ota_get_running_partition().as_ref() }?;
    Some(running.subtype)
}

/// Set the verbosity of the device's own (serial) logging. Levels above the compile-time maximum
/// (`CONFIG_LOG_MAXIMUM_LEVEL`, Info by default) have no further effect.
pub fn set_log_level(level: LogLevel) {
    let l = match level {
        LogLevel::Error => sys::esp_log_level_t_ESP_LOG_ERROR,
        LogLevel::Warn => sys::esp_log_level_t_ESP_LOG_WARN,
        LogLevel::Info => sys::esp_log_level_t_ESP_LOG_INFO,
        LogLevel::Debug => sys::esp_log_level_t_ESP_LOG_DEBUG,
    };
    // SAFETY: "*" is a valid NUL-terminated tag pattern.
    unsafe { sys::esp_log_level_set(b"*\0".as_ptr().cast(), l) }
}

/// `(sum of idle-task runtime, total runtime)` since boot, both in the same timer ticks.
fn runtime_counters() -> Option<(u32, u32)> {
    // SAFETY: we pass a buffer of `capacity` TaskStatus_t and FreeRTOS fills at most that many.
    unsafe {
        let capacity = sys::uxTaskGetNumberOfTasks() as usize + 4; // headroom for tasks spawned meanwhile
        let mut tasks: Vec<TaskStatus_t> = Vec::with_capacity(capacity);
        let mut total: u32 = 0;
        let got =
            sys::uxTaskGetSystemState(tasks.as_mut_ptr(), capacity as u32, &mut total) as usize;
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
        let Some((idle, total)) = runtime_counters() else {
            return self.last_load.get();
        };
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

    fn fill_random(&mut self, buf: &mut [u8]) {
        // SAFETY: writes exactly `buf.len()` bytes into `buf`. Hardware RNG (true random while
        // the radio is on, which it always is here).
        unsafe { sys::esp_fill_random(buf.as_mut_ptr().cast(), buf.len()) }
    }

    fn restart(&mut self) {
        // SAFETY: never returns; the chip reboots.
        unsafe { sys::esp_restart() }
    }

    fn nvs_usage(&self) -> (u32, u32) {
        let mut stats = sys::nvs_stats_t::default();
        // SAFETY: a null partition name means the default "nvs" partition; `stats` is a valid out-pointer.
        let rc = unsafe { sys::nvs_get_stats(core::ptr::null(), &mut stats) };
        if rc != sys::ESP_OK {
            return (0, 0);
        }
        (
            stats.used_entries as u32 * NVS_ENTRY_BYTES,
            stats.free_entries as u32 * NVS_ENTRY_BYTES,
        )
    }

    fn partition_info(&self) -> PartitionInfo {
        let running = running_slot();
        let slot = |subtype| {
            ota_partition(subtype).map_or((false, 0), |p| (running == Some(p.subtype), p.size))
        };
        let (ota0_active, ota0_size) =
            slot(sys::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_APP_OTA_0);
        let (ota1_active, ota1_size) =
            slot(sys::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_APP_OTA_1);
        PartitionInfo {
            ota0_active,
            ota1_active,
            ota0_size,
            ota1_size,
        }
    }

    fn build_facts(&self) -> BuildFacts {
        BuildFacts {
            version: env!("CARGO_PKG_VERSION"),
            git_sha: option_env!("BEEWAVE_GIT_SHA").unwrap_or("unknown"),
            built: option_env!("BEEWAVE_BUILT").unwrap_or("unknown"),
            ota_key_fp: self.ota_key_fp,
            request_conn_params: super::ble_gatt::REQUEST_CONN_PARAMS,
            coex_debug: cfg!(feature = "coex_debug"),
            // Bonding is intentionally off (see ble_gatt/pairing.rs); optional hardening.
            bonding: false,
        }
    }

    fn self_test_nvs(&mut self) -> bool {
        let Ok(mut nvs) = EspNvs::new(self.nvs.clone(), SELFTEST_NAMESPACE, true) else {
            return false;
        };
        const PROBE: u32 = 0xB33_BEE;
        let wrote = nvs.set_u32("probe", PROBE).is_ok();
        let read = nvs.get_u32("probe").ok().flatten() == Some(PROBE);
        let erased = nvs.remove("probe").is_ok();
        wrote && read && erased
    }

    fn self_test_ble(&self) -> bool {
        // SAFETY: plain NimBLE host queries; non-zero means an advertise / connection procedure is active.
        unsafe { sys::ble_gap_adv_active() != 0 || sys::ble_gap_conn_active() != 0 }
    }

    fn self_test_ota_slot(&self) -> bool {
        let in_slot = matches!(
            running_slot(),
            Some(sys::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_APP_OTA_0)
                | Some(sys::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_APP_OTA_1)
        );
        // SAFETY: plain query; a coherent table always has another slot to update into.
        let other = unsafe { !sys::esp_ota_get_next_update_partition(core::ptr::null()).is_null() };
        in_slot && other
    }
}
