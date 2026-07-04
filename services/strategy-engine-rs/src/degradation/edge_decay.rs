use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct EdgeDecayTracker;

impl EdgeDecayTracker {
    /// Evaluate edge decay: returns true if the ratio of current expectancy
    /// to baseline expectancy falls below the threshold (default 70%).
    pub fn detect_decay(
        baseline_expectancy: Decimal,
        current_expectancy: Decimal,
        threshold: Decimal,
    ) -> bool {
        if baseline_expectancy <= Decimal::ZERO {
            return false;
        }

        let ratio = current_expectancy / baseline_expectancy;
        ratio < threshold
    }
}
