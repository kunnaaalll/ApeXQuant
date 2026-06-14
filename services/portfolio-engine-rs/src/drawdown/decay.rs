use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownRecoveryModel {
    pub half_life_periods: u32,
    pub stability_threshold: f64,
    pub current_recovery_score: f64,
    pub consecutive_positive_periods: u32,
}

impl DrawdownRecoveryModel {
    pub fn new(half_life_periods: u32, stability_threshold: f64) -> Self {
        Self {
            half_life_periods,
            stability_threshold,
            current_recovery_score: 0.0,
            consecutive_positive_periods: 0,
        }
    }

    pub fn update(&mut self, is_positive_period: bool, magnitude: f64) -> f64 {
        if is_positive_period {
            self.consecutive_positive_periods += 1;
            
            // Gradual recovery using an exponential moving average or similar decay function
            let alpha = 1.0 - (-std::f64::consts::LN_2 / self.half_life_periods as f64).exp();
            let target_score = 1.0; // The goal is full recovery (1.0)
            
            // Scale recovery speed by magnitude (clamped between 0.1 and 2.0)
            let speed_multiplier = magnitude.clamp(0.1, 2.0);
            self.current_recovery_score += alpha * speed_multiplier * (target_score - self.current_recovery_score);
        } else {
            // Instant penalization on negative periods to prevent false recovery
            self.consecutive_positive_periods = 0;
            self.current_recovery_score *= 0.5; // Halve the score on a down day
        }

        self.current_recovery_score
    }

    pub fn is_stable(&self) -> bool {
        self.current_recovery_score >= self.stability_threshold
    }
}
