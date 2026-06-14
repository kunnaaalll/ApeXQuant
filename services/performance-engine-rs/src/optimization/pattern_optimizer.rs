use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternState {
    Elite,
    Strong,
    Average,
    Weak,
    Disabled,
}

#[derive(Debug, Clone)]
pub struct PatternOptimizer {
    pub min_elite_expectancy: Decimal,
    pub min_strong_expectancy: Decimal,
    pub min_average_expectancy: Decimal,
    pub max_disabled_drawdown: Decimal,
    pub min_sample_quality: Decimal,
}

impl PatternOptimizer {
    pub fn new(min_elite: Decimal, min_strong: Decimal, min_avg: Decimal, max_dd: Decimal, min_quality: Decimal) -> Self {
        Self {
            min_elite_expectancy: min_elite,
            min_strong_expectancy: min_strong,
            min_average_expectancy: min_avg,
            max_disabled_drawdown: max_dd,
            min_sample_quality: min_quality,
        }
    }

    pub fn evaluate(&self, expectancy: Decimal, drawdown: Decimal, sample_quality: Decimal) -> PatternState {
        if drawdown >= self.max_disabled_drawdown {
            return PatternState::Disabled;
        }

        if sample_quality < self.min_sample_quality {
            return PatternState::Average; // default state if insufficient sample
        }

        if expectancy >= self.min_elite_expectancy {
            PatternState::Elite
        } else if expectancy >= self.min_strong_expectancy {
            PatternState::Strong
        } else if expectancy >= self.min_average_expectancy {
            PatternState::Average
        } else {
            PatternState::Weak
        }
    }
}
