#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatencyState {
    Healthy,
    Warning,
    Restricted,
    Critical,
    Frozen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatencyGuards {
    pub broker_latency_ms: u32,
    pub exchange_latency_ms: u32,
    pub network_latency_ms: u32,
}

impl LatencyGuards {
    pub fn new(broker_latency_ms: u32, exchange_latency_ms: u32, network_latency_ms: u32) -> Self {
        Self {
            broker_latency_ms,
            exchange_latency_ms,
            network_latency_ms,
        }
    }

    pub fn total_latency_ms(&self) -> u32 {
        self.broker_latency_ms
            .saturating_add(self.exchange_latency_ms)
            .saturating_add(self.network_latency_ms)
    }

    pub fn get_state(&self) -> LatencyState {
        let total = self.total_latency_ms();

        if total < 20 {
            LatencyState::Healthy
        } else if total < 50 {
            LatencyState::Warning
        } else if total < 100 {
            LatencyState::Restricted
        } else if total < 200 {
            LatencyState::Critical
        } else {
            LatencyState::Frozen
        }
    }

    pub fn get_score(&self) -> u32 {
        let total = self.total_latency_ms();

        if total <= 10 {
            0
        } else if total >= 200 {
            100
        } else {
            // Map 10ms-200ms to 0-100 score
            let diff = total - 10;
            let score = (diff * 100) / 190;
            score.min(100)
        }
    }
}
