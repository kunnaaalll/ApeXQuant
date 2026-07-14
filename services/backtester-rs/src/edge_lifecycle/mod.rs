use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeState {
    Emerging,
    Growing,
    Stable,
    Weakening,
    Dying,
    Retired,
}

#[derive(Debug, Clone)]
pub struct EdgeLifecycle {
    pub strategy_id: Uuid,
    pub state: EdgeState,
    pub winrate_drift: Decimal,
    pub expectancy_drift: Decimal,
    pub drawdown_change: Decimal,
    pub last_regime_shift: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl EdgeLifecycle {
    pub fn new(strategy_id: Uuid) -> Self {
        Self {
            strategy_id,
            state: EdgeState::Emerging,
            winrate_drift: Decimal::ZERO,
            expectancy_drift: Decimal::ZERO,
            drawdown_change: Decimal::ZERO,
            last_regime_shift: None,
            updated_at: Utc::now(),
        }
    }

    pub fn update_drift(&mut self, winrate: Decimal, expectancy: Decimal, drawdown: Decimal) {
        self.winrate_drift = winrate;
        self.expectancy_drift = expectancy;
        self.drawdown_change = drawdown;
        self.updated_at = Utc::now();
        self.evaluate_state();
    }

    fn evaluate_state(&mut self) {
        // Deterministic state transition logic based on drift limits
        if self.expectancy_drift < Decimal::ZERO {
            self.state = EdgeState::Weakening;
        }
    }
}
