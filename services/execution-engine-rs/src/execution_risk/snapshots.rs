use super::events::ExecutionRiskEvent;
use super::circuit_breaker::ExecutionProtectionState;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct ExecutionRiskSnapshot {
    pub current_state: ExecutionProtectionState,
    pub consecutive_rejections: u32,
    pub failure_score: u32,
    pub total_latency_ms: u32,
    pub spread_multiplier: Decimal,
}

impl Default for ExecutionRiskSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionRiskSnapshot {
    pub fn new() -> Self {
        Self {
            current_state: ExecutionProtectionState::Normal,
            consecutive_rejections: 0,
            failure_score: 0,
            total_latency_ms: 0,
            spread_multiplier: Decimal::ONE,
        }
    }

    pub fn apply_event(&mut self, event: &ExecutionRiskEvent) {
        match event {
            ExecutionRiskEvent::StateTransition { to, .. } => {
                self.current_state = *to;
            }
            ExecutionRiskEvent::SpreadChanged { spread_multiplier, .. } => {
                self.spread_multiplier = *spread_multiplier;
            }
            ExecutionRiskEvent::LatencyChanged { latency_score: _, network_latency_ms, exchange_latency_ms, broker_latency_ms } => {
                self.total_latency_ms = broker_latency_ms.saturating_add(*exchange_latency_ms).saturating_add(*network_latency_ms);
            }
            ExecutionRiskEvent::FailureRecorded { failure_score, .. } => {
                self.failure_score = *failure_score;
            }
            ExecutionRiskEvent::RejectionRecorded { consecutive_rejections, .. } => {
                self.consecutive_rejections = *consecutive_rejections;
            }
            _ => {}
        }
    }

    pub fn rebuild_from_events(events: &[ExecutionRiskEvent]) -> Self {
        let mut snapshot = Self::new();
        for event in events {
            snapshot.apply_event(event);
        }
        snapshot
    }
}
