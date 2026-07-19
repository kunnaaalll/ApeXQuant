use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvolutionState {
    Improving,
    Stable,
    Weakening,
    Collapsing,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyEvolutionAssessment {
    pub expectancy_drift: Decimal,
    pub drawdown_drift: Decimal,
    pub confidence_drift: Decimal,
    pub stability_drift: Decimal,
    pub state: EvolutionState,
}

impl StrategyEvolutionAssessment {
    pub fn new(
        expectancy_drift: Decimal,
        drawdown_drift: Decimal,
        confidence_drift: Decimal,
        stability_drift: Decimal,
    ) -> Self {
        let state = Self::determine_state(
            expectancy_drift,
            drawdown_drift,
            confidence_drift,
            stability_drift,
        );
        Self {
            expectancy_drift,
            drawdown_drift,
            confidence_drift,
            stability_drift,
            state,
        }
    }

    fn determine_state(
        expectancy_drift: Decimal,
        drawdown_drift: Decimal,
        confidence_drift: Decimal,
        stability_drift: Decimal,
    ) -> EvolutionState {
        use rust_decimal_macros::dec;

        let expectancy_collapse_threshold = dec!(-0.20);
        let drawdown_collapse_threshold = dec!(0.20);
        let confidence_collapse_threshold = dec!(-0.30);

        if expectancy_drift <= expectancy_collapse_threshold
            || drawdown_drift >= drawdown_collapse_threshold
            || confidence_drift <= confidence_collapse_threshold
        {
            return EvolutionState::Collapsing;
        }

        let improving_threshold = dec!(0.05);
        if expectancy_drift >= improving_threshold
            && drawdown_drift <= dec!(0)
            && stability_drift >= dec!(0)
        {
            return EvolutionState::Improving;
        }

        let weakening_threshold = dec!(-0.05);
        if expectancy_drift <= weakening_threshold || drawdown_drift >= improving_threshold {
            return EvolutionState::Weakening;
        }

        EvolutionState::Stable
    }
}
