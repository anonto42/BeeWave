//! Coexistence diagnostics for the BLE notification path.
//!
//! With the `coex_debug` feature this records, per notification, how long the message waited in the
//! telemetry queue and how long the `notify` call took, and logs percentiles every `REPORT_EVERY`.
//! Without the feature `CoexDiag` is an empty type whose methods do nothing.
#[cfg(feature = "coex_debug")]
mod enabled {
    use std::time::{Duration, Instant};

    use beewave_core::features::latency::LatencyHistogram;

    /// How often the summary is logged.
    const REPORT_EVERY: Duration = Duration::from_secs(10);

    pub struct CoexDiag {
        queue: LatencyHistogram,
        notify: LatencyHistogram,
        last_report: Instant,
    }

    impl CoexDiag {
        pub fn new() -> Self {
            Self {
                queue: LatencyHistogram::new(),
                notify: LatencyHistogram::new(),
                last_report: Instant::now(),
            }
        }

        pub fn record(&mut self, queued: Duration, notify: Duration) {
            self.queue.record(micros(queued));
            self.notify.record(micros(notify));
            if self.last_report.elapsed() >= REPORT_EVERY {
                self.report();
            }
        }

        fn report(&mut self) {
            for (name, h) in [("queue", &self.queue), ("notify", &self.notify)] {
                if let Some(s) = h.summary() {
                    log::info!(
                        "coex {name}: n={} p50<={}us p99<={}us max={}us",
                        s.count,
                        s.p50_us,
                        s.p99_us,
                        s.max_us
                    );
                }
            }
            self.queue.reset();
            self.notify.reset();
            self.last_report = Instant::now();
        }
    }

    fn micros(d: Duration) -> u32 {
        u32::try_from(d.as_micros()).unwrap_or(u32::MAX)
    }
}

#[cfg(feature = "coex_debug")]
pub use enabled::CoexDiag;

#[cfg(not(feature = "coex_debug"))]
mod disabled {
    use std::time::Duration;

    pub struct CoexDiag;

    impl CoexDiag {
        pub fn new() -> Self {
            Self
        }

        pub fn record(&mut self, _queued: Duration, _notify: Duration) {}
    }
}

#[cfg(not(feature = "coex_debug"))]
pub use disabled::CoexDiag;
