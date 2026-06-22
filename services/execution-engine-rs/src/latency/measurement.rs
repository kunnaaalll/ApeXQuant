#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatencyMeasurement {
    pub broker_latency_ms: u64,
    pub exchange_latency_ms: u64,
    pub network_latency_ms: u64,
}

impl LatencyMeasurement {
    pub fn new(broker_latency_ms: u64, exchange_latency_ms: u64, network_latency_ms: u64) -> Self {
        Self {
            broker_latency_ms,
            exchange_latency_ms,
            network_latency_ms,
        }
    }

    pub fn total_ms(&self) -> u64 {
        self.broker_latency_ms + self.exchange_latency_ms + self.network_latency_ms
    }
}
