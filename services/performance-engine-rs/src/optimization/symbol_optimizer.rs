use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolState {
    Elite,
    Strong,
    Average,
    Weak,
    Forbidden,
}

#[derive(Debug, Clone)]
pub struct SymbolOptimizer {
    pub min_elite_expectancy: Decimal,
    pub min_strong_expectancy: Decimal,
    pub min_average_expectancy: Decimal,
    pub max_forbidden_drawdown: Decimal,
}

impl SymbolOptimizer {
    pub fn new(min_elite: Decimal, min_strong: Decimal, min_avg: Decimal, max_forbidden_dd: Decimal) -> Self {
        Self {
            min_elite_expectancy: min_elite,
            min_strong_expectancy: min_strong,
            min_average_expectancy: min_avg,
            max_forbidden_drawdown: max_forbidden_dd,
        }
    }

    pub fn evaluate(&self, expectancy: Decimal, drawdown: Decimal, confidence: Decimal) -> SymbolState {
        if drawdown >= self.max_forbidden_drawdown {
            return SymbolState::Forbidden;
        }

        if confidence < rust_decimal_macros::dec!(0.5) {
            return SymbolState::Average; // Need more data
        }

        if expectancy >= self.min_elite_expectancy {
            SymbolState::Elite
        } else if expectancy >= self.min_strong_expectancy {
            SymbolState::Strong
        } else if expectancy >= self.min_average_expectancy {
            SymbolState::Average
        } else {
            SymbolState::Weak
        }
    }
}
