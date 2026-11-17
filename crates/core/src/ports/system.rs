/// Where the two update slots are and which one is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartitionInfo {
    pub ota0_active: bool,
    pub ota1_active: bool,
    pub ota0_size: u32,
    pub ota1_size: u32,
}

/// Facts fixed when the firmware was built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildFacts {
    pub version: &'static str,
    pub git_sha: &'static str,
    pub built: &'static str,
    /// First 16 bytes of the SHA-256 of the embedded OTA public key; zeros when there is none.
    pub ota_key_fp: [u8; 16],
    pub request_conn_params: bool,
    pub coex_debug: bool,
    pub bonding: bool,
}

/// Device-level facts and actions.
pub trait System {
    fn free_heap(&self) -> u32;
    fn cpu_load(&self) -> u8;
    fn restart(&mut self);
    /// Fill `buf` with cryptographically secure random bytes (hardware RNG on the device).
    fn fill_random(&mut self, buf: &mut [u8]);
    /// Bytes of NVS in use and free.
    fn nvs_usage(&self) -> (u32, u32);
    fn partition_info(&self) -> PartitionInfo;
    fn build_facts(&self) -> BuildFacts;
    /// Self-test: write, read back and erase a scratch key in a reserved NVS namespace.
    fn self_test_nvs(&mut self) -> bool;
    /// Self-test: the BLE stack is advertising or connected.
    fn self_test_ble(&self) -> bool;
    /// Self-test: the running slot is ota_0 or ota_1 and the update bookkeeping is coherent.
    fn self_test_ota_slot(&self) -> bool;
}
