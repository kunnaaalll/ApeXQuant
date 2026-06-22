use super::spread_guards::SpreadGuards;
use super::latency_guards::LatencyGuards;
use super::fill_quality_guards::FillQualityGuards;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CooldownEngine {
    pub stable_cycles_required: u32,
    pub stable_cycles_completed: u32,
    pub fills_required: u32,
    pub fills_completed: u32,
}

impl CooldownEngine {
    pub fn new(stable_cycles_required: u32, fills_required: u32) -> Self {
        Self {
            stable_cycles_required,
            stable_cycles_completed: 0,
            fills_required,
            fills_completed: 0,
        }
    }

    pub fn record_cycle(&mut self, spread: &SpreadGuards, latency: &LatencyGuards, fills: &FillQualityGuards) {
        let is_healthy = spread.get_state() == super::circuit_breaker::ExecutionProtectionState::Normal
            && latency.get_state() == super::latency_guards::LatencyState::Healthy
            && fills.get_grade() == super::fill_quality_guards::FillGrade::Elite;

        if is_healthy {
            self.stable_cycles_completed = self.stable_cycles_completed.saturating_add(1);
        } else {
            self.stable_cycles_completed = 0;
        }
    }

    pub fn record_fill(&mut self) {
        self.fills_completed = self.fills_completed.saturating_add(1);
    }

    pub fn reset(&mut self) {
        self.stable_cycles_completed = 0;
        self.fills_completed = 0;
    }

    pub fn is_ready_for_recovery(&self) -> bool {
        self.stable_cycles_completed >= self.stable_cycles_required
            && self.fills_completed >= self.fills_required
    }
}
