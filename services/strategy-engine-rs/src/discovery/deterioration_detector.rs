use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeteriorationState {
    Healthy = 0,
    Caution = 1,
    Danger = 2,
    Critical = 3,
    Collapse = 4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeteriorationDetector {
    state: DeteriorationState,
}

impl DeteriorationDetector {
    pub fn new() -> Self {
        Self {
            state: DeteriorationState::Healthy,
        }
    }

    pub fn update(
        &mut self,
        expectancy_collapse: Decimal,
        confidence_degradation: Decimal,
        drawdown_increase: Decimal,
        streak_deterioration: Decimal,
    ) -> DeteriorationState {
        // Higher values denote worse state
        let total_risk =
            expectancy_collapse + confidence_degradation + drawdown_increase + streak_deterioration;

        let target_state = if total_risk >= dec!(0.40) {
            DeteriorationState::Collapse
        } else if total_risk >= dec!(0.30) {
            DeteriorationState::Critical
        } else if total_risk >= dec!(0.20) {
            DeteriorationState::Danger
        } else if total_risk >= dec!(0.10) {
            DeteriorationState::Caution
        } else {
            DeteriorationState::Healthy
        };

        // Immediate downgrade allowed
        if target_state > self.state {
            self.state = target_state;
        } else if target_state < self.state {
            // Gradual recovery
            let current_val = self.state as u8;
            if current_val > 0 {
                self.state = match current_val - 1 {
                    0 => DeteriorationState::Healthy,
                    1 => DeteriorationState::Caution,
                    2 => DeteriorationState::Danger,
                    3 => DeteriorationState::Critical,
                    _ => DeteriorationState::Collapse,
                };
            }
        }

        self.state
    }

    pub fn state(&self) -> DeteriorationState {
        self.state
    }
}

impl Default for DeteriorationDetector {
    fn default() -> Self {
        Self::new()
    }
}
