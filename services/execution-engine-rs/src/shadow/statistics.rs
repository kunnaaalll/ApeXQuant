use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowStatistics {
    pub exact_match_count: u64,
    pub close_match_count: u64,
    pub warning_count: u64,
    pub mismatch_count: u64,
    pub critical_mismatch_count: u64,
    
    pub daily_match_rate: Decimal,
    pub weekly_match_rate: Decimal,
    pub monthly_match_rate: Decimal,
}

impl ShadowStatistics {
    pub const fn new() -> Self {
        Self {
            exact_match_count: 0,
            close_match_count: 0,
            warning_count: 0,
            mismatch_count: 0,
            critical_mismatch_count: 0,
            
            daily_match_rate: dec!(100),
            weekly_match_rate: dec!(100),
            monthly_match_rate: dec!(100),
        }
    }

    pub fn total_matches(&self) -> u64 {
        self.exact_match_count
            .saturating_add(self.close_match_count)
            .saturating_add(self.warning_count)
            .saturating_add(self.mismatch_count)
            .saturating_add(self.critical_mismatch_count)
    }

    pub fn match_percentage(&self) -> Decimal {
        let total = self.total_matches();
        if total == 0 {
            return dec!(100);
        }

        let good_matches = self.exact_match_count.saturating_add(self.close_match_count);
        let percent = (Decimal::from(good_matches) / Decimal::from(total)) * dec!(100);
        
        if percent > dec!(100) {
            dec!(100)
        } else if percent < dec!(0) {
            dec!(0)
        } else {
            percent
        }
    }
}

impl Default for ShadowStatistics {
    fn default() -> Self {
        Self::new()
    }
}
