use crate::circuit_breaker::events::CircuitBreakerEvent;
use crate::circuit_breaker::CircuitBreakerState;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CircuitBreakerSnapshot {
    pub state: CircuitBreakerState,
    pub version: u64,
    pub timestamp_ms: u64,
    pub remaining_drawdown_capacity: Decimal,
    pub severity_score: Decimal,
}

impl CircuitBreakerSnapshot {
    pub fn new(timestamp_ms: u64) -> Self {
        Self {
            state: CircuitBreakerState::Normal,
            version: 0,
            timestamp_ms,
            remaining_drawdown_capacity: Decimal::ZERO,
            severity_score: Decimal::ZERO,
        }
    }

    pub fn apply_event(&mut self, event: &CircuitBreakerEvent) {
        match event {
            CircuitBreakerEvent::StateChanged {
                to,
                timestamp_ms,
                version,
                ..
            } => {
                self.state = *to;
                self.timestamp_ms = *timestamp_ms;
                self.version = *version;
            }
            CircuitBreakerEvent::SeverityScoreUpdated {
                new_score,
                timestamp_ms,
                version,
            } => {
                self.severity_score = *new_score;
                self.timestamp_ms = *timestamp_ms;
                self.version = *version;
            }
            CircuitBreakerEvent::DrawdownCapacityUpdated {
                remaining_capacity,
                timestamp_ms,
                version,
            } => {
                self.remaining_drawdown_capacity = *remaining_capacity;
                self.timestamp_ms = *timestamp_ms;
                self.version = *version;
            }
        }
    }

    pub fn replay(events: &[CircuitBreakerEvent], initial_timestamp_ms: u64) -> Self {
        let mut snapshot = Self::new(initial_timestamp_ms);
        for event in events {
            snapshot.apply_event(event);
        }
        snapshot
    }
}
