use std::thread;

/// Spawn a thread with no core affinity (FreeRTOS `tskNO_AFFINITY`): the scheduler may run it
/// on whichever of the two cores is free. Stacks come from the heap, so keep workers few.
pub fn spawn_worker<F>(name: &str, stack_size: usize, f: F) -> thread::JoinHandle<()>
where
    F: FnOnce() + Send + 'static,
{
    thread::Builder::new()
        .name(name.into())
        .stack_size(stack_size)
        .spawn(f)
        .expect("spawn worker") // startup only: no memory for a thread means we cannot run at all
}
