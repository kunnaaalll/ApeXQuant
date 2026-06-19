use crate::shadow::comparison::ComparisonState;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Default)]
pub struct StatisticsWindow {
    pub total_comparisons: u64,
    pub exact_matches: u64,
    pub close_matches: u64,
    pub warnings: u64,
    pub mismatches: u64,
    pub critical_failures: u64,
}

impl StatisticsWindow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, state: &ComparisonState) {
        self.total_comparisons += 1;
        match state {
            ComparisonState::ExactMatch => self.exact_matches += 1,
            ComparisonState::CloseMatch => self.close_matches += 1,
            ComparisonState::Warning => self.warnings += 1,
            ComparisonState::Mismatch => self.mismatches += 1,
            ComparisonState::Critical => self.critical_failures += 1,
        }
    }

    pub fn agreement_percentage(&self) -> Decimal {
        if self.total_comparisons == 0 {
            return Decimal::new(100, 0); // Default to 100% agreement when empty
        }

        let acceptable = self.exact_matches + self.close_matches;
        let acceptable_dec = Decimal::from(acceptable);
        let total_dec = Decimal::from(self.total_comparisons);
        let hundred = Decimal::new(100, 0);

        let mut percentage = (acceptable_dec / total_dec) * hundred;
        percentage = percentage.round_dp(2);

        if percentage > hundred {
            hundred
        } else if percentage < Decimal::ZERO {
            Decimal::ZERO
        } else {
            percentage
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StatisticsEngine {
    pub daily: StatisticsWindow,
    pub weekly: StatisticsWindow,
    pub monthly: StatisticsWindow,
}

impl StatisticsEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, state: &ComparisonState) {
        self.daily.record(state);
        self.weekly.record(state);
        self.monthly.record(state);
    }
}
