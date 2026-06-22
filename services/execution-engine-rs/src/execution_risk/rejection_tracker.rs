use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use super::circuit_breaker::ExecutionProtectionState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionState {
    Normal,
    Elevated,
    Danger,
    Locked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectionTracker {
    pub consecutive_rejections: u32,
    pub rolling_rejection_rate: Decimal,
}

impl RejectionTracker {
    pub fn new(consecutive_rejections: u32, rolling_rejection_rate: Decimal) -> Self {
        Self {
            consecutive_rejections,
            rolling_rejection_rate,
        }
    }

    pub fn get_rejection_state(&self) -> RejectionState {
        if self.consecutive_rejections >= 5 || self.rolling_rejection_rate >= dec!(0.2) {
            RejectionState::Locked
        } else if self.consecutive_rejections >= 3 || self.rolling_rejection_rate >= dec!(0.1) {
            RejectionState::Danger
        } else if self.consecutive_rejections >= 1 || self.rolling_rejection_rate >= dec!(0.05) {
            RejectionState::Elevated
        } else {
            RejectionState::Normal
        }
    }

    pub fn get_protection_state(&self) -> ExecutionProtectionState {
        match self.get_rejection_state() {
            RejectionState::Normal => ExecutionProtectionState::Normal,
            RejectionState::Elevated => ExecutionProtectionState::Warning,
            RejectionState::Danger => ExecutionProtectionState::Critical,
            RejectionState::Locked => ExecutionProtectionState::Frozen,
        }
    }
}
