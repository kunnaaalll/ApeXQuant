use std::time::Duration;
use backoff::backoff::Backoff;

pub struct ExecutionBackoff {
    inner: backoff::ExponentialBackoff,
}

impl ExecutionBackoff {
    pub fn new(initial: Duration, max: Duration, multiplier: f64) -> Self {
        let mut eb = backoff::ExponentialBackoff::default();
        eb.initial_interval = initial;
        eb.max_interval = max;
        eb.multiplier = multiplier;
        eb.randomization_factor = 0.5; // Full jitter randomization
        
        Self { inner: eb }
    }

    pub fn next_delay(&mut self) -> Option<Duration> {
        self.inner.next_backoff()
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }
}
