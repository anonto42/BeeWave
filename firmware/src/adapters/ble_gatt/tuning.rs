//! BLE link tuning knobs. Wi-Fi sniffing and BLE share one radio, so these trade notification
//! latency against airtime and power. **Off by default** (`REQUEST_CONN_PARAMS = false`): the phone
//! chooses the parameters. Turn it on only while measuring on the board; units follow the BLE spec.
use esp32_nimble::BLEServer;

/// Ask the phone to use the parameters below after it connects. The phone may ignore the request.
pub const REQUEST_CONN_PARAMS: bool = false;
/// Connection interval bounds in 1.25 ms units (24 = 30 ms, 40 = 50 ms).
pub const CONN_INTERVAL_MIN: u16 = 24;
pub const CONN_INTERVAL_MAX: u16 = 40;
/// Connection events the device may skip. 0 keeps notification latency lowest.
pub const SLAVE_LATENCY: u16 = 0;
/// Supervision timeout in 10 ms units (400 = 4 s). Must exceed `(1 + latency) * max interval * 2`.
pub const SUPERVISION_TIMEOUT: u16 = 400;

/// Apply the knobs to a fresh connection (no-op unless `REQUEST_CONN_PARAMS`).
pub fn request(server: &mut BLEServer, conn_handle: u16) {
    if !REQUEST_CONN_PARAMS {
        return;
    }
    if let Err(e) = server.update_conn_params(
        conn_handle,
        CONN_INTERVAL_MIN,
        CONN_INTERVAL_MAX,
        SLAVE_LATENCY,
        SUPERVISION_TIMEOUT,
    ) {
        log::warn!("connection parameter request failed: {e:?}");
    }
}
