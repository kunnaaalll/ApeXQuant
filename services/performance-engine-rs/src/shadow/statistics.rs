use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShadowStatistics {
    pub total_evaluations: u64,
    pub exact_matches: u64,
    pub close_matches: u64,
    pub warnings: u64,
    pub mismatches: u64,
    pub critical_failures: u64,
    pub max_drift: Decimal,
    pub cumulative_drift: Decimal,
}

impl ShadowStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_result(
        &mut self,
        state: super::comparison::ShadowComparisonState,
        max_difference: Decimal,
    ) {
        self.total_evaluations += 1;
        match state {
            super::comparison::ShadowComparisonState::ExactMatch => self.exact_matches += 1,
            super::comparison::ShadowComparisonState::CloseMatch => self.close_matches += 1,
            super::comparison::ShadowComparisonState::Warning => self.warnings += 1,
            super::comparison::ShadowComparisonState::Mismatch => self.mismatches += 1,
            super::comparison::ShadowComparisonState::Critical => self.critical_failures += 1,
        }

        if max_difference > self.max_drift {
            self.max_drift = max_difference;
        }
        self.cumulative_drift += max_difference;
    }

    pub fn get_overall_agreement_percentage(&self) -> Decimal {
        if self.total_evaluations == 0 {
            return Decimal::new(100, 0);
        }

        let positive_matches = self.exact_matches + self.close_matches;
        let percentage = (Decimal::from(positive_matches) / Decimal::from(self.total_evaluations))
            * Decimal::new(100, 0);
        percentage.round_dp(4)
    }

    pub fn get_average_drift(&self) -> Decimal {
        if self.total_evaluations == 0 {
            return Decimal::ZERO;
        }

        let avg = self.cumulative_drift / Decimal::from(self.total_evaluations);
        avg.round_dp(4)
    }
}
