use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender};
use std::time::{Duration, Instant};

use beewave_protocol::{
    decode_command, encode_telemetry, DeviceCommand, TelemetryMessage, MAX_BODY,
};
use esp32_nimble::{BLEAdvertisementData, BLEDevice, NimbleProperties};

use super::coex_diag::CoexDiag;
use super::names::{command_name, notable_telemetry};
use super::{tuning, uuids};
use crate::runtime::{Event, Timed};

/// Set once the NimBLE device exists and advertising has started. `BLEDevice::take()` lazily initialises a
/// `static mut` without synchronisation, so any other thread that needs the device must wait for this.
static READY: AtomicBool = AtomicBool::new(false);

/// Has the BLE stack been brought up (safe for other threads to call `BLEDevice::take()`)?
pub fn is_ready() -> bool {
    READY.load(Ordering::Acquire)
}

/// Preferred ATT MTU (must be >= beewave_protocol::MIN_MTU).
const PREFERRED_MTU: u16 = 247;

/// Runs forever: GATT server + notification pump.
///
/// SECURITY: BLE bonding is intentionally OFF (see `pairing.rs`). Commands are protected by the
/// application-level HMAC handshake in the core (`features::auth`): the link is not encrypted,
/// but no command runs and no status/scan telemetry is sent until the client proves the key.
pub fn run(
    events: SyncSender<Event>,
    tele_rx: Receiver<Timed<TelemetryMessage>>,
) -> anyhow::Result<()> {
    let device = BLEDevice::take();
    device.set_preferred_mtu(PREFERRED_MTU)?;

    let server = device.get_server();
    // Every connect and disconnect ends the previous session. Blocking `send` on purpose: a
    // dropped reset would let the next client inherit an authenticated session.
    let on_connect_events = events.clone();
    server.on_connect(move |server, desc| {
        log::info!("BLE connected: {:?}", desc);
        tuning::request(server, desc.conn_handle());
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
                if !matches!(cmd, DeviceCommand::OtaChunk { .. }) {
                    log::info!("rx command: {}", command_name(&cmd));
                }
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
    READY.store(true, Ordering::Release);

    let mut seq: u8 = 0;
    let mut buf = [0u8; MAX_BODY + 2];
    let mut diag = CoexDiag::new();
    loop {
        match tele_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Timed { queued_at, msg }) => {
                if server.connected_count() == 0 {
                    continue; // nobody listening: drop instead of queueing
                }
                if let Some(name) = notable_telemetry(&msg) {
                    log::info!("tx telemetry: {name}");
                }
                if let Ok(n) = encode_telemetry(seq, &msg, &mut buf) {
                    seq = seq.wrapping_add(1);
                    let queued = queued_at.elapsed();
                    let started = Instant::now();
                    telemetry.lock().set_value(&buf[..n]).notify();
                    diag.record(queued, started.elapsed());
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
    }
}
