use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::models::AllocationState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AllocationRecoveryModel {
    pub drawdown_threshold: Decimal,
    pub current_drawdown: Decimal,
    pub recovery_decay_rate: Decimal,
    pub recovery_progress: Decimal, // 0.0 to 1.0 (1.0 = fully recovered)
    pub is_in_drawdown: bool,
}

impl AllocationRecoveryModel {
    pub fn new(drawdown_threshold: Decimal, recovery_decay_rate: Decimal) -> Self {
        Self {
            drawdown_threshold,
            current_drawdown: Decimal::ZERO,
            recovery_decay_rate,
            recovery_progress: Decimal::ONE,
            is_in_drawdown: false,
        }
    }

    pub fn update_drawdown(&mut self, drawdown: Decimal) {
        self.current_drawdown = drawdown;
        if self.current_drawdown >= self.drawdown_threshold {
            self.is_in_drawdown = true;
            self.recovery_progress = Decimal::ZERO;
        }
    }

    pub fn tick_decay(&mut self) {
        if self.is_in_drawdown && self.current_drawdown < self.drawdown_threshold {
            // Slowly recover
            self.recovery_progress += self.recovery_decay_rate;
            if self.recovery_progress >= Decimal::ONE {
                self.recovery_progress = Decimal::ONE;
                self.is_in_drawdown = false;
            }
        }
    }

    pub fn recommend_state(&self, base_state: AllocationState) -> AllocationState {
        if self.is_in_drawdown {
            if self.recovery_progress < Decimal::new(25, 2) {
                AllocationState::Conservative // Deep in drawdown
            } else if self.recovery_progress < Decimal::new(75, 2) {
                AllocationState::Recovery
            } else {
                AllocationState::Defensive
            }
        } else {
            // Even if not strictly in drawdown, if base state is Aggressive but we just recovered,
            // we could temper it, but that logic might be handled by the heat state.
            base_state
        }
    }
}
