use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeState {
    Emerging,
    Learning,
    Growing,
    Stable,
    Decaying,
    Failing,
    Retired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftMetrics {
    pub expectancy_drift: Decimal,
    pub win_rate_drift: Decimal,
    pub drawdown_drift: Decimal,
    pub regime_dependency_shift: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeLifecycle {
    pub strategy_id: String,
    pub state: EdgeState,
    pub drift_metrics: DriftMetrics,
    pub last_evaluated: u64,
}

pub struct EdgeLifecycleEngine {}

impl EdgeLifecycleEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate_drift(
        &self,
        current_metrics: &DriftMetrics,
        historical_baseline: &DriftMetrics,
    ) -> DriftMetrics {
        // Compute the difference between current and historical metrics
        DriftMetrics {
            expectancy_drift: current_metrics.expectancy_drift - historical_baseline.expectancy_drift,
            win_rate_drift: current_metrics.win_rate_drift - historical_baseline.win_rate_drift,
            drawdown_drift: current_metrics.drawdown_drift - historical_baseline.drawdown_drift,
            regime_dependency_shift: current_metrics.regime_dependency_shift - historical_baseline.regime_dependency_shift,
        }
    }

    pub fn transition_state(&self, lifecycle: &mut EdgeLifecycle, new_drift: &DriftMetrics) {
        // State machine logic for edge lifecycle
        // E.g., if expectancy_drift is highly negative, move towards Decaying/Failing
        
        let decay_threshold = Decimal::new(-10, 2); // -0.10
        let failing_threshold = Decimal::new(-20, 2); // -0.20
        
        if new_drift.expectancy_drift <= failing_threshold {
            lifecycle.state = EdgeState::Failing;
        } else if new_drift.expectancy_drift <= decay_threshold {
            lifecycle.state = EdgeState::Decaying;
        } else if lifecycle.state == EdgeState::Growing && new_drift.expectancy_drift >= Decimal::ZERO {
            lifecycle.state = EdgeState::Stable;
        }
        
        lifecycle.drift_metrics = new_drift.clone();
        // Update last_evaluated timestamp in a real system
    }
}

impl Default for EdgeLifecycleEngine {
    fn default() -> Self {
        Self::new()
    }
}
