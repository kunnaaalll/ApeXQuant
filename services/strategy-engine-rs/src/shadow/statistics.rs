use rust_decimal::Decimal;
use crate::shadow::comparison::ShadowComparisonState;
use std::cmp;

#[derive(Debug, Clone)]
pub struct StatisticsEngine {
    pub daily_exact_matches: u64,
    pub daily_close_matches: u64,
    pub daily_warnings: u64,
    pub daily_mismatches: u64,

    pub weekly_exact_matches: u64,
    pub weekly_close_matches: u64,
    pub weekly_warnings: u64,
    pub weekly_mismatches: u64,

    pub monthly_exact_matches: u64,
    pub monthly_close_matches: u64,
    pub monthly_warnings: u64,
    pub monthly_mismatches: u64,
}

impl Default for StatisticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StatisticsEngine {
    pub fn new() -> Self {
        Self {
            daily_exact_matches: 0,
            daily_close_matches: 0,
            daily_warnings: 0,
            daily_mismatches: 0,
            weekly_exact_matches: 0,
            weekly_close_matches: 0,
            weekly_warnings: 0,
            weekly_mismatches: 0,
            monthly_exact_matches: 0,
            monthly_close_matches: 0,
            monthly_warnings: 0,
            monthly_mismatches: 0,
        }
    }

    pub fn record(&mut self, state: ShadowComparisonState) {
        match state {
            ShadowComparisonState::ExactMatch => {
                self.daily_exact_matches += 1;
                self.weekly_exact_matches += 1;
                self.monthly_exact_matches += 1;
            }
            ShadowComparisonState::CloseMatch => {
                self.daily_close_matches += 1;
                self.weekly_close_matches += 1;
                self.monthly_close_matches += 1;
            }
            ShadowComparisonState::Warning => {
                self.daily_warnings += 1;
                self.weekly_warnings += 1;
                self.monthly_warnings += 1;
            }
            ShadowComparisonState::Mismatch | ShadowComparisonState::Critical => {
                self.daily_mismatches += 1;
                self.weekly_mismatches += 1;
                self.monthly_mismatches += 1;
            }
        }
    }

    pub fn match_percentage(&self) -> Decimal {
        let total = self.daily_exact_matches 
            + self.daily_close_matches 
            + self.daily_warnings 
            + self.daily_mismatches;
            
        if total == 0 {
            return Decimal::ZERO;
        }

        let exact = Decimal::from(self.daily_exact_matches);
        let tot = Decimal::from(total);

        let percentage = (exact / tot) * Decimal::new(100, 0);
        
        let zero = Decimal::ZERO;
        let hundred = Decimal::new(100, 0);
        
        cmp::min(cmp::max(percentage, zero), hundred)
    }
}
