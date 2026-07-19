use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ShadowComparisonState {
    #[default]
    ExactMatch,
    CloseMatch,
    Warning,
    Mismatch,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShadowComparisonResult {
    pub agreement_percentage: Decimal,
    pub mismatch_count: u64,
    pub average_difference: Decimal,
    pub maximum_difference: Decimal,
    pub state: ShadowComparisonState,
}

impl ShadowComparisonResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn determine_state(&mut self, tolerance: Decimal, critical_threshold: Decimal) {
        if self.maximum_difference == Decimal::ZERO && self.mismatch_count == 0 {
            self.state = ShadowComparisonState::ExactMatch;
        } else if self.maximum_difference <= tolerance {
            self.state = ShadowComparisonState::CloseMatch;
        } else if self.maximum_difference <= critical_threshold {
            self.state = ShadowComparisonState::Warning;
        } else {
            if self.agreement_percentage < Decimal::new(95, 2) {
                self.state = ShadowComparisonState::Critical;
            } else {
                self.state = ShadowComparisonState::Mismatch;
            }
        }
    }
}
