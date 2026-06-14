use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegimeState {
    HighlyFavored,
    Favored,
    Neutral,
    Avoid,
    Forbidden,
}

#[derive(Debug, Clone)]
pub struct RegimeOptimizer {
    pub min_trades: u32,
    pub min_highly_favored_expectancy: Decimal,
    pub min_favored_expectancy: Decimal,
    pub min_confidence: Decimal,
}

impl RegimeOptimizer {
    pub fn new(min_trades: u32, min_highly: Decimal, min_favored: Decimal, min_conf: Decimal) -> Self {
        Self {
            min_trades,
            min_highly_favored_expectancy: min_highly,
            min_favored_expectancy: min_favored,
            min_confidence: min_conf,
        }
    }

    pub fn evaluate(&self, trades: u32, expectancy: Decimal, confidence: Decimal) -> RegimeState {
        if trades < self.min_trades || confidence < self.min_confidence {
            return RegimeState::Neutral;
        }

        if expectancy < rust_decimal_macros::dec!(0.0) {
            if expectancy < rust_decimal_macros::dec!(-1.0) {
                return RegimeState::Forbidden;
            }
            return RegimeState::Avoid;
        }

        if expectancy >= self.min_highly_favored_expectancy {
            RegimeState::HighlyFavored
        } else if expectancy >= self.min_favored_expectancy {
            RegimeState::Favored
        } else {
            RegimeState::Neutral
        }
    }
}
