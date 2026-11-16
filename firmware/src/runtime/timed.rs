use std::time::Instant;

/// A message stamped with the moment it was queued, so the consumer can measure how long it waited.
/// The stamp costs one `Instant` per queue slot; it is only *reported* with the `coex_debug` feature.
pub struct Timed<T> {
    pub queued_at: Instant,
    pub msg: T,
}

impl<T> Timed<T> {
    pub fn now(msg: T) -> Self {
        Self {
            queued_at: Instant::now(),
            msg,
        }
    }
}
