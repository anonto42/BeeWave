//! Runtime plumbing: threads, the event channel and the app task.
//! Not domain logic (that is `walpad_core`) and not a hardware adapter (that is `adapters/`).
mod app_task;
mod channels;
mod cpu;
mod event;
mod worker;

pub use app_task::run_app;
pub use channels::channels;
pub use cpu::core_id;
pub use event::Event;
pub use worker::spawn_worker;
