use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Adjusts raw expectancy or metrics into a normalized 0-100 score.
#[derive(Debug, Clone)]
pub struct ScoreAdjuster {
    pub min_expectancy: Decimal,
    pub max_expectancy: Decimal,
}

impl ScoreAdjuster {
    pub fn new(min_expectancy: Decimal, max_expectancy: Decimal) -> Self {
        assert!(min_expectancy < max_expectancy, "Min must be less than Max");
        Self {
            min_expectancy,
            max_expectancy,
        }
    }

    pub fn normalize(&self, value: Decimal) -> Decimal {
        if value <= self.min_expectancy {
            return dec!(0.0);
        }
        if value >= self.max_expectancy {
            return dec!(100.0);
        }

        let range = self.max_expectancy - self.min_expectancy;
        let diff = value - self.min_expectancy;
        (diff / range) * dec!(100.0)
    }
}
