use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone)]
pub struct AdaptiveWeights {
    pub recent_weight: Decimal,
    pub historical_weight: Decimal,
    pub learning_rate: Decimal,
}

impl AdaptiveWeights {
    pub fn new(initial_recent_weight: Decimal, initial_historical_weight: Decimal, learning_rate: Decimal) -> Self {
        Self {
            recent_weight: initial_recent_weight,
            historical_weight: initial_historical_weight,
            learning_rate,
        }
    }

    pub fn update_weights(&mut self, is_regime_shifting: bool) {
        if is_regime_shifting {
            self.recent_weight += self.learning_rate;
            self.historical_weight -= self.learning_rate;
        } else {
            let diff = self.recent_weight - dec!(0.5);
            self.recent_weight -= diff * self.learning_rate;
            self.historical_weight = dec!(1.0) - self.recent_weight;
        }

        if self.recent_weight > dec!(0.9) {
            self.recent_weight = dec!(0.9);
            self.historical_weight = dec!(0.1);
        }
        if self.recent_weight < dec!(0.1) {
            self.recent_weight = dec!(0.1);
            self.historical_weight = dec!(0.9);
        }
    }
}
