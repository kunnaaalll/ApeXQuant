use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct BenchmarkEngine {
    pub average_latency: Decimal,
    pub p99_latency: Decimal,
}

impl Default for BenchmarkEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchmarkEngine {
    pub fn new() -> Self {
        Self {
            average_latency: Decimal::ZERO,
            p99_latency: Decimal::ZERO,
        }
    }

    pub fn track(&mut self, latencies: &[Decimal]) {
        if latencies.is_empty() {
            return;
        }

        let mut sum = Decimal::ZERO;
        let mut sorted = latencies.to_vec();
        sorted.sort();

        for l in &sorted {
            sum += *l;
        }

        self.average_latency = sum / Decimal::from(sorted.len());

        let p99_index = (sorted.len() * 99) / 100;
        self.p99_latency = sorted[p99_index];
    }

    pub fn is_within_target(&self) -> bool {
        self.average_latency < Decimal::new(2, 0) && self.p99_latency < Decimal::new(10, 0)
    }
}
