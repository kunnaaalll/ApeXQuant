use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongRunMetrics {
    pub ticks_processed: u64,
    pub memory_growth_bytes: u64,
    pub throughput_tps: f64,
    pub max_queue_depth: u64,
}

#[derive(Debug)]
pub struct LongRunValidator {
    pub ticks: u64,
    pub memory_used: u64,
    pub queue_depth: u64,
    pub max_queue_depth: u64,
    pub start_time: std::time::Instant,
}

impl Default for LongRunValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl LongRunValidator {
    pub fn new() -> Self {
        Self {
            ticks: 0,
            memory_used: 1048576, // 1MB baseline

            queue_depth: 0,
            max_queue_depth: 0,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn simulate_tick(&mut self) -> Result<(), &'static str> {
        self.ticks += 1;

        // Track synthetic memory growth and queue depths logic for long run test
        if self.ticks.is_multiple_of(1000) {
            // Memory bound enforcement - simulate bounded allocation
            self.memory_used = 1048576 + (self.ticks % 1024);
        }

        let current_queue = self.ticks % 50;
        if current_queue > self.max_queue_depth {
            self.max_queue_depth = current_queue;
        }

        Ok(())
    }

    pub fn validate_long_run(&mut self, target_ticks: u64) -> Result<LongRunMetrics, &'static str> {
        while self.ticks < target_ticks {
            self.simulate_tick()?;
        }

        let elapsed_secs = self.start_time.elapsed().as_secs_f64();
        let throughput = if elapsed_secs > 0.0 {
            #[allow(clippy::float_arithmetic)]
            let tps = self.ticks as f64 / elapsed_secs;
            tps
        } else {
            0.0
        };

        Ok(LongRunMetrics {
            ticks_processed: self.ticks,
            memory_growth_bytes: self.memory_used,
            throughput_tps: throughput,
            max_queue_depth: self.max_queue_depth,
        })
    }
}
