use beewave_protocol::TelemetryMessage;

use super::*;
use crate::ports::{BuildFacts, PartitionInfo, System};

struct FakeSystem;

impl System for FakeSystem {
    fn free_heap(&self) -> u32 {
        1234
    }
    fn cpu_load(&self) -> u8 {
        7
    }
    fn restart(&mut self) {}
    fn fill_random(&mut self, _: &mut [u8]) {}
    fn nvs_usage(&self) -> (u32, u32) {
        (0, 0)
    }
    fn partition_info(&self) -> PartitionInfo {
        PartitionInfo {
            ota0_active: true,
            ota1_active: false,
            ota0_size: 0,
            ota1_size: 0,
        }
    }
    fn build_facts(&self) -> BuildFacts {
        BuildFacts {
            version: "",
            git_sha: "",
            built: "",
            ota_key_fp: [0; 16],
            request_conn_params: false,
            coex_debug: false,
            bonding: false,
        }
    }
    fn self_test_nvs(&mut self) -> bool {
        true
    }
    fn self_test_ble(&self) -> bool {
        true
    }
    fn self_test_ota_slot(&self) -> bool {
        true
    }
}

#[test]
fn builds_status() {
    assert_eq!(
        sample(&FakeSystem, 3, 2, true),
        TelemetryMessage::Status {
            active_channel: 3,
            free_heap: 1234,
            cpu_load: 7,
            dropped: 2,
            scanning: true
        }
    );
}
