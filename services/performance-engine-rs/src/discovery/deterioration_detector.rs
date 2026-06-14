use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeteriorationState {
    Healthy,
    Weakening,
    Warning,
    Critical,
    Collapse,
}

#[derive(Debug, Clone)]
pub struct DeteriorationDetector {
    pub max_drawdown_threshold: Decimal,
    pub min_expectancy_threshold: Decimal,
}

impl DeteriorationDetector {
    pub fn new(max_drawdown_threshold: Decimal, min_expectancy_threshold: Decimal) -> Self {
        Self {
            max_drawdown_threshold,
            min_expectancy_threshold,
        }
    }

    pub fn detect(&self, current_drawdown: Decimal, expectancy: Decimal, stability: Decimal) -> DeteriorationState {
        // Absolute limits trigger collapse
        if current_drawdown >= self.max_drawdown_threshold || expectancy <= self.min_expectancy_threshold {
            return DeteriorationState::Collapse;
        }

        // Warning state logic
        if current_drawdown >= self.max_drawdown_threshold * rust_decimal_macros::dec!(0.8) || expectancy <= self.min_expectancy_threshold * rust_decimal_macros::dec!(1.5) {
            return DeteriorationState::Critical;
        }

        if current_drawdown >= self.max_drawdown_threshold * rust_decimal_macros::dec!(0.5) || stability < rust_decimal_macros::dec!(0.5) {
            return DeteriorationState::Warning;
        }

        if stability < rust_decimal_macros::dec!(0.8) || expectancy < self.min_expectancy_threshold * rust_decimal_macros::dec!(3.0) {
            return DeteriorationState::Weakening;
        }

        DeteriorationState::Healthy
    }
}
