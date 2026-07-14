use core::time::Duration;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct LatencyContext {
    pub order_time: OffsetDateTime,
    pub is_high_load: bool,
}

pub struct LatencyModel {
    pub base_network_latency: Duration,
    pub base_broker_latency: Duration,
    pub base_exchange_latency: Duration,
}

impl LatencyModel {
    pub fn new(
        base_network_latency: Duration,
        base_broker_latency: Duration,
        base_exchange_latency: Duration,
    ) -> Self {
        Self {
            base_network_latency,
            base_broker_latency,
            base_exchange_latency,
        }
    }

    pub fn total_latency(&self, ctx: &LatencyContext) -> Duration {
        let mut total =
            self.base_network_latency + self.base_broker_latency + self.base_exchange_latency;

        if ctx.is_high_load {
            // Under high load, latency doubles deterministically.
            total *= 2;
        }
        total
    }
}
