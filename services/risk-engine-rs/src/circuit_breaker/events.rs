use rust_decimal::Decimal;
use crate::circuit_breaker::CircuitBreakerState;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CircuitBreakerEvent {
    StateChanged {
        from: CircuitBreakerState,
        to: CircuitBreakerState,
        timestamp_ms: u64,
        version: u64,
        reason: String,
    },
    SeverityScoreUpdated {
        new_score: Decimal,
        timestamp_ms: u64,
        version: u64,
    },
    DrawdownCapacityUpdated {
        remaining_capacity: Decimal,
        timestamp_ms: u64,
        version: u64,
    },
}
