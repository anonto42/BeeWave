//! Runtime plumbing (threads + event channel). Not domain logic, not an adapter.
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;
use walpad_core::features::scan::Observation;
use walpad_protocol::{DeviceCommand, TelemetryMessage};

/// Everything the app task reacts to. Bounded so a stalled consumer can't eat the heap.
pub enum Event {
    Command(DeviceCommand),
    Frame(Observation),
}

pub struct Channels {
    pub events_tx: SyncSender<Event>,
    pub events_rx: Receiver<Event>,
    pub tele_tx: SyncSender<TelemetryMessage>,
    pub tele_rx: Receiver<TelemetryMessage>,
}

pub fn channels() -> Channels {
    let (events_tx, events_rx) = sync_channel(32);
    let (tele_tx, tele_rx) = sync_channel(32);
    Channels { events_tx, events_rx, tele_tx, tele_rx }
}

/// Unpinned worker (tskNO_AFFINITY). Stacks come from the heap: keep workers few.
pub fn spawn_worker<F: FnOnce() + Send + 'static>(name: &str, stack: usize, f: F) -> thread::JoinHandle<()> {
    thread::Builder::new().name(name.into()).stack_size(stack).spawn(f).expect("spawn worker")
}

pub fn core_id() -> i32 {
    unsafe { esp_idf_svc::sys::xPortGetCoreID() as i32 }
}
