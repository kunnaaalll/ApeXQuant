use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecoveryState {
    Broken = 0,
    Recovering = 1,
    Stable = 2,
    Healthy = 3,
}

#[derive(Debug, Clone)]
pub struct RiskRecoveryModel {
    pub score: Decimal, // 0 to 1
    pub state: RecoveryState,
    pub consecutive_positive_periods: u64,
    pub required_positive_periods: u64,
    pub decay_factor: Decimal, // e.g., 0.9 for exponential decay on loss
}

impl RiskRecoveryModel {
    pub fn new(required_positive_periods: u64, decay_factor: Decimal) -> Self {
        Self {
            score: Decimal::ZERO,
            state: RecoveryState::Broken,
            consecutive_positive_periods: 0,
            required_positive_periods,
            decay_factor,
        }
    }

    pub fn tick(&mut self, is_loss: bool) {
        if is_loss {
            // Single negative period resets progress drastically
            self.consecutive_positive_periods = 0;
            self.score *= self.decay_factor;
            self.update_state();
            return;
        }

        self.consecutive_positive_periods += 1;

        if self.consecutive_positive_periods >= self.required_positive_periods {
            // Gradual recovery
            let recovery_step = Decimal::new(10, 2); // 0.10
            self.score += recovery_step;
            if self.score > Decimal::ONE {
                self.score = Decimal::ONE;
            }
        }

        self.update_state();
    }

    fn update_state(&mut self) {
        // Enforce sequential transitions implicitly by only moving state step-by-step
        // based on score ranges, but remember: no instant jump from Broken -> Healthy
        let target_state = if self.score >= Decimal::new(90, 2) {
            // 0.90
            RecoveryState::Healthy
        } else if self.score >= Decimal::new(60, 2) {
            // 0.60
            RecoveryState::Stable
        } else if self.score >= Decimal::new(30, 2) {
            // 0.30
            RecoveryState::Recovering
        } else {
            RecoveryState::Broken
        };

        // If target is much better, only advance one step at a time
        let current_level = self.state as i8;
        let target_level = target_state as i8;

        if target_level > current_level {
            // Only improve by 1 level maximum
            self.state = match current_level + 1 {
                1 => RecoveryState::Recovering,
                2 => RecoveryState::Stable,
                3 => RecoveryState::Healthy,
                _ => self.state,
            };
        } else {
            // Downgrades can be immediate
            self.state = target_state;
        }
    }
}
