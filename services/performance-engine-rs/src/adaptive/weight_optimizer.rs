use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeightOptimizer {
    pub min_weight: Decimal,
    pub max_weight: Decimal,
    pub max_step: Decimal, // maximum change per period
}

impl Default for WeightOptimizer {
    fn default() -> Self {
        Self {
            min_weight: dec!(0.5),
            max_weight: dec!(2.0),
            max_step: dec!(0.05),
        }
    }
}

impl WeightOptimizer {
    pub fn new(min_weight: Decimal, max_weight: Decimal, max_step: Decimal) -> Self {
        assert!(min_weight < max_weight, "Min must be less than Max");
        assert!(max_step > dec!(0.0), "Step must be positive");
        Self {
            min_weight,
            max_weight,
            max_step,
        }
    }

    pub fn optimize(&self, current_weight: Decimal, target_weight: Decimal) -> Decimal {
        let mut new_weight = current_weight;

        let diff = target_weight - current_weight;
        if diff > self.max_step {
            new_weight += self.max_step;
        } else if diff < -self.max_step {
            new_weight -= self.max_step;
        } else {
            new_weight = target_weight;
        }

        if new_weight < self.min_weight {
            self.min_weight
        } else if new_weight > self.max_weight {
            self.max_weight
        } else {
            new_weight
        }
    }
}
