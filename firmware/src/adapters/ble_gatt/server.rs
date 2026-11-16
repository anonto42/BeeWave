use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender};
use std::time::Duration;

use esp32_nimble::enums::{AuthReq, SecurityIOCap};
use esp32_nimble::{BLEAdvertisementData, BLEDevice, NimbleProperties};
use walpad_protocol::{decode_command, encode_telemetry, TelemetryMessage, MAX_BODY};

use super::uuids;
use crate::runtime::Event;

/// Preferred ATT MTU (must be >= walpad_protocol::MIN_MTU).
const PREFERRED_MTU: u16 = 247;

/// Runs forever: GATT server + notification pump.
pub fn run(events: SyncSender<Event>, tele_rx: Receiver<TelemetryMessage>) -> anyhow::Result<()> {
    let device = BLEDevice::take();
    device.set_preferred_mtu(PREFERRED_MTU)?;
    device
        .security()
        .set_auth(AuthReq::Bond | AuthReq::Sc)
        .set_io_cap(SecurityIOCap::NoInputNoOutput); // "Just Works"; upgrade to a passkey later

    let server = device.get_server();
    server.on_connect(|_server, desc| log::info!("BLE connected: {:?}", desc));
    server.on_disconnect(|_desc, reason| log::info!("BLE disconnected: {:?}", reason));

    let service = server.create_service(uuids::SERVICE);

    // Commands: write-without-response, only over an encrypted + authenticated link.
    let command = service.lock().create_characteristic(
        uuids::COMMAND,
        NimbleProperties::WRITE_NO_RSP | NimbleProperties::WRITE_ENC | NimbleProperties::WRITE_AUTHEN,
    );
    command.lock().on_write(move |args| match decode_command(args.recv_data()) {
        Ok((_seq, cmd)) => {
            if events.try_send(Event::Command(cmd)).is_err() {
                log::warn!("event queue full, command dropped");
            }
        }
        Err(e) => log::warn!("bad command: {e:?}"),
    });

    // Telemetry: notify-only.
    let telemetry = service.lock().create_characteristic(uuids::TELEMETRY, NimbleProperties::NOTIFY);

    let advertising = device.get_advertising();
    advertising
        .lock()
        .set_data(BLEAdvertisementData::new().name(uuids::DEVICE_NAME).add_service_uuid(uuids::SERVICE))?;
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
