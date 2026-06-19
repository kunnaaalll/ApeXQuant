use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightState {
    Stable,
    Increasing,
    Decreasing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeightType {
    Symbol,
    Regime,
    Session,
    Timeframe,
    Pattern,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightOptimizer {
    weight: Decimal,
    state: WeightState,
}

impl WeightOptimizer {
    pub fn new(initial_weight: Decimal) -> Self {
        Self {
            weight: Self::clamp_weight(initial_weight),
            state: WeightState::Stable,
        }
    }

    pub fn update(&mut self, target_weight: Decimal) {
        let target = Self::clamp_weight(target_weight);
        let max_shift = dec!(0.05);

        let delta = target - self.weight;

        if delta > dec!(0.0) {
            let shift = std::cmp::min(delta, max_shift);
            self.weight += shift;
            self.state = WeightState::Increasing;
        } else if delta < dec!(0.0) {
            let shift = std::cmp::max(delta, -max_shift);
            self.weight += shift;
            self.state = WeightState::Decreasing;
        } else {
            self.state = WeightState::Stable;
        }

        self.weight = Self::clamp_weight(self.weight);
    }

    pub fn weight(&self) -> Decimal {
        self.weight
    }

    pub fn state(&self) -> WeightState {
        self.state
    }

    fn clamp_weight(weight: Decimal) -> Decimal {
        let min = dec!(0.50);
        let max = dec!(2.00);
        if weight < min {
            min
        } else if weight > max {
            max
        } else {
            weight
        }
    }
}
