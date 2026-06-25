use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthGrade {
    Healthy,
    Warning,
    Critical,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub grade: HealthGrade,
    pub queue_depth: u64,
    pub subscriber_lag: u64,
    pub replay_lag: u64,
    pub throughput: Decimal, // Events per second
    pub latency_ms: Decimal,
}

impl Default for SystemHealth {
    fn default() -> Self {
        Self {
            grade: HealthGrade::Healthy,
            queue_depth: 0,
            subscriber_lag: 0,
            replay_lag: 0,
            throughput: Decimal::ZERO,
            latency_ms: Decimal::ZERO,
        }
    }
}
