//! Runtime plumbing: threads, the event channel and the app task.
//! Not domain logic (that is `beewave_core`) and not a hardware adapter (that is `adapters/`).
mod app_task;
mod channels;
mod cpu;
mod event;
mod timed;
mod worker;

pub use app_task::{run_app, Hooks};
pub use channels::channels;
pub use cpu::core_id;
pub use event::Event;
pub use timed::Timed;
pub use worker::spawn_worker;
