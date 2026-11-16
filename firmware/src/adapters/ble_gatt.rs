//! BLE GATT adapter (NimBLE). Driving: command writes -> Event::Command. Driven: TelemetrySink -> notify.
//! NOTE: written against esp32-nimble 0.11; verify on first build.
use std::sync::mpsc::{Receiver, SyncSender, TrySendError};
use std::time::Duration;

use esp32_nimble::enums::{AuthReq, SecurityIOCap};
use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};
use walpad_core::ports::TelemetrySink;
use walpad_protocol::{decode_command, encode_telemetry, TelemetryMessage, MAX_BODY};

use crate::runtime::Event;

/// TelemetrySink handed to the core: non-blocking, lossy (never stalls the app task).
pub struct BleSink(pub SyncSender<TelemetryMessage>);

impl TelemetrySink for BleSink {
    fn publish(&mut self, msg: TelemetryMessage) -> bool {
        !matches!(self.0.try_send(msg), Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)))
    }
}

/// Runs forever: GATT server + notify pump.
pub fn run(events: SyncSender<Event>, tele_rx: Receiver<TelemetryMessage>) -> anyhow::Result<()> {
    let device = BLEDevice::take();
    device.set_preferred_mtu(247)?;
    device.security().set_auth(AuthReq::Bond | AuthReq::Sc).set_io_cap(SecurityIOCap::NoInputNoOutput);

    let server = device.get_server();
    server.on_connect(|_s, d| log::info!("BLE connected: {:?}", d));
    server.on_disconnect(|_d, r| log::info!("BLE disconnected: {:?}", r));

    let uuid = uuid128!("7a1c0001-9d3e-4b8a-a1f0-5e5b1c0de001");
    let service = server.create_service(uuid);

    let cmd = service.lock().create_characteristic(
        uuid128!("7a1c0002-9d3e-4b8a-a1f0-5e5b1c0de001"),
        NimbleProperties::WRITE_NO_RSP | NimbleProperties::WRITE_ENC | NimbleProperties::WRITE_AUTHEN,
    );
    cmd.lock().on_write(move |args| match decode_command(args.recv_data()) {
        Ok((_seq, c)) => {
            if events.try_send(Event::Command(c)).is_err() { log::warn!("event queue full, command dropped"); }
        }
        Err(e) => log::warn!("bad command: {e:?}"),
    });

    let tele = service.lock().create_characteristic(
        uuid128!("7a1c0003-9d3e-4b8a-a1f0-5e5b1c0de001"),
        NimbleProperties::NOTIFY,
    );

    let adv = device.get_advertising();
    adv.lock().set_data(BLEAdvertisementData::new().name("WalPad").add_service_uuid(uuid))?;
    adv.lock().start()?;

    let (mut seq, mut buf) = (0u8, [0u8; MAX_BODY + 2]);
    loop {
        match tele_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(msg) => {
                if server.connected_count() == 0 { continue; }
                if let Ok(n) = encode_telemetry(seq, &msg, &mut buf) {
                    seq = seq.wrapping_add(1);
                    tele.lock().set_value(&buf[..n]).notify();
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => return Ok(()),
        }
    }
}
