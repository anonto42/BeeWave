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
/// SECURITY: BLE bonding is intentionally OFF (see `pairing.rs`). Commands are protected by the
/// application-level HMAC handshake in the core (`features::auth`): the link is not encrypted,
/// but no command runs and no status/scan telemetry is sent until the client proves the key.
pub fn run(events: SyncSender<Event>, tele_rx: Receiver<TelemetryMessage>) -> anyhow::Result<()> {
    let device = BLEDevice::take();
    device.set_preferred_mtu(PREFERRED_MTU)?;

    let server = device.get_server();
    // Every connect and disconnect ends the previous session. Blocking `send` on purpose: a
    // dropped reset would let the next client inherit an authenticated session.
    let on_connect_events = events.clone();
    server.on_connect(move |_server, desc| {
        log::info!("BLE connected: {:?}", desc);
        let _ = on_connect_events.send(Event::LinkReset);
    });
    let on_disconnect_events = events.clone();
    server.on_disconnect(move |_desc, reason| {
        log::info!("BLE disconnected: {:?}", reason);
        let _ = on_disconnect_events.send(Event::LinkReset);
    });

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
