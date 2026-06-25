use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use rust_decimal::MathematicalOps;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownRecoveryModel {
    pub half_life_periods: u32,
    pub stability_threshold: Decimal,
    pub current_recovery_score: Decimal,
    pub consecutive_positive_periods: u32,
}

impl DrawdownRecoveryModel {
    pub fn new(half_life_periods: u32, stability_threshold: Decimal) -> Self {
        Self {
            half_life_periods,
            stability_threshold,
            current_recovery_score: Decimal::ZERO,
            consecutive_positive_periods: 0,
        }
    }

    pub fn update(&mut self, is_positive_period: bool, magnitude: Decimal) -> Decimal {
        if is_positive_period {
            self.consecutive_positive_periods += 1;
            
            // Gradual recovery using an exponential moving average or similar decay function
            let ln2 = Decimal::new(6931471805599453, 16); // 0.6931471805599453
            let half_life_dec = Decimal::from(self.half_life_periods);
            let exponent = -ln2 / half_life_dec;
            
            let alpha = Decimal::ONE - exponent.exp();
            let target_score = Decimal::ONE; // The goal is full recovery (1.0)
            
            // Scale recovery speed by magnitude (clamped between 0.1 and 2.0)
            let min_mag = Decimal::new(1, 1); // 0.1
            let max_mag = Decimal::new(2, 0); // 2.0
            let speed_multiplier = magnitude.clamp(min_mag, max_mag);
            
            self.current_recovery_score += alpha * speed_multiplier * (target_score - self.current_recovery_score);
        } else {
            // Instant penalization on negative periods to prevent false recovery
            self.consecutive_positive_periods = 0;
            let half = Decimal::new(5, 1); // 0.5
            self.current_recovery_score *= half; // Halve the score on a down day
        }

        self.current_recovery_score
    }

    pub fn is_stable(&self) -> bool {
        self.current_recovery_score >= self.stability_threshold
    }
}
