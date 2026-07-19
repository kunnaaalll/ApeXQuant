use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct CorrelationCollapseEngine {
    base_correlation: Decimal,
}

impl CorrelationCollapseEngine {
    pub fn new(base_correlation: Decimal) -> Self {
        Self {
            base_correlation: base_correlation.clamp(dec!(-1.0), dec!(1.0)),
        }
    }

    pub fn apply_collapse(&self, multiplier: Decimal) -> Decimal {
        let delta = dec!(1.0) - self.base_correlation;
        // The higher the multiplier, the closer it gets to 1.0.
        // Prevent division by zero safely if multiplier <= 0
        let effective_multiplier = if multiplier <= dec!(0.0) {
            dec!(1.0)
        } else {
            multiplier
        };
        let shift = delta * (dec!(1.0) - (dec!(1.0) / effective_multiplier));
        let mut new_corr = self.base_correlation + shift;
        if new_corr > dec!(1.0) {
            new_corr = dec!(1.0);
        }
        new_corr
    }
}
