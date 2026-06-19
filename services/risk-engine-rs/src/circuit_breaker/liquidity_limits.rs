use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LiquidityState {
    Healthy = 0,
    Watch = 1,
    Warning = 2,
    Danger = 3,
    Frozen = 4,
}

#[derive(Debug, Clone)]
pub struct LiquidityLimitAssessment {
    pub spread_expansion: Decimal,
    pub slippage: Decimal,
    pub execution_degradation: Decimal,
    pub state: LiquidityState,
}

impl LiquidityLimitAssessment {
    pub fn new(
        spread_expansion: Decimal,
        slippage: Decimal,
        execution_degradation: Decimal,
    ) -> Self {
        let mut assessment = Self {
            spread_expansion,
            slippage,
            execution_degradation,
            state: LiquidityState::Healthy,
        };
        assessment.update_state();
        assessment
    }

    fn update_state(&mut self) {
        let max_metric = self.spread_expansion
            .max(self.slippage)
            .max(self.execution_degradation);

        // Assume threshold values as percentages or multipliers
        // e.g. > 10.0x degradation = Frozen
        self.state = if max_metric > Decimal::new(100, 1) { // > 10.0
            LiquidityState::Frozen
        } else if max_metric > Decimal::new(50, 1) { // > 5.0
            LiquidityState::Danger
        } else if max_metric > Decimal::new(30, 1) { // > 3.0
            LiquidityState::Warning
        } else if max_metric > Decimal::new(15, 1) { // > 1.5
            LiquidityState::Watch
        } else {
            LiquidityState::Healthy
        };
    }
}
