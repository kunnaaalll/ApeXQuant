use rust_decimal::Decimal;
use std::collections::HashMap;

pub struct ParameterComparisonEngine;

impl ParameterComparisonEngine {
    /// Compare two parameter sets and calculate divergence
    pub fn calculate_divergence(
        set_a: &HashMap<String, Decimal>,
        set_b: &HashMap<String, Decimal>,
    ) -> Decimal {
        let mut sum_diff = Decimal::ZERO;
        let mut count = 0;

        for (key, val_a) in set_a {
            if let Some(val_b) = set_b.get(key) {
                sum_diff += (val_a - val_b).abs();
                count += 1;
            }
        }

        if count == 0 {
            Decimal::ZERO
        } else {
            sum_diff / Decimal::from(count)
        }
    }
}
