use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender};
use std::time::Duration;

use beewave_protocol::{decode_command, encode_telemetry, TelemetryMessage, MAX_BODY};
use esp32_nimble::{BLEAdvertisementData, BLEDevice, NimbleProperties};

use super::uuids;
use crate::runtime::Event;

/// Preferred ATT MTU (must be >= beewave_protocol::MIN_MTU).
const PREFERRED_MTU: u16 = 247;

/// Runs forever: GATT server + notification pump.
///
/// SECURITY: pairing/bonding is intentionally OFF right now (see `pairing.rs`). Android's system
/// pairing screen does not reliably bond with a plain custom GATT peripheral, and Web Bluetooth
/// does not trigger it either, so an encrypted characteristic here just hung the link with no
/// usable error. This is safe only on your own bench: re-enable `pairing::require()` before this
/// device is ever used anywhere someone else could write to it over BLE.
pub fn run(events: SyncSender<Event>, tele_rx: Receiver<TelemetryMessage>) -> anyhow::Result<()> {
    let device = BLEDevice::take();
    device.set_preferred_mtu(PREFERRED_MTU)?;

    let server = device.get_server();
    server.on_connect(|_server, desc| log::info!("BLE connected: {:?}", desc));
    server.on_disconnect(|_desc, reason| log::info!("BLE disconnected: {:?}", reason));

    let service = server.create_service(uuids::SERVICE);

    // Commands: write-with-response (so a rejection is visible to the client), no pairing required.
    let command = service
        .lock()
        .create_characteristic(uuids::COMMAND, NimbleProperties::WRITE);
    command
        .lock()
        .on_write(move |args| match decode_command(args.recv_data()) {
            Ok((_seq, cmd)) => {
                if events.try_send(Event::Command(cmd)).is_err() {
                    log::warn!("event queue full, command dropped");
                }
            }
            Err(e) => log::warn!("bad command: {e:?}"),
        });

    // Telemetry: notify-only.
    let telemetry = service
        .lock()
        .create_characteristic(uuids::TELEMETRY, NimbleProperties::NOTIFY);

    let advertising = device.get_advertising();
    advertising.lock().set_data(
        BLEAdvertisementData::new()
            .name(uuids::DEVICE_NAME)
            .add_service_uuid(uuids::SERVICE),
    )?;
    advertising.lock().start()?;

    let mut seq: u8 = 0;
    let mut buf = [0u8; MAX_BODY + 2];
    loop {
        match tele_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(msg) => {
                if server.connected_count() == 0 {
                    continue; // nobody listening: drop instead of queueing
                }
                if let Ok(n) = encode_telemetry(seq, &msg, &mut buf) {
                    seq = seq.wrapping_add(1);
                    telemetry.lock().set_value(&buf[..n]).notify();
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
    }
}
