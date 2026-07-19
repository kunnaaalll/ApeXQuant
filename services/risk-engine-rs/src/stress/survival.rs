use super::leverage::LeverageState;
use super::liquidity::LiquidityState;
use super::volatility::VolatilityState;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurvivalState {
    Excellent,
    Strong,
    Moderate,
    Weak,
    Critical,
    Failed,
}

pub struct SurvivalEngine;

impl SurvivalEngine {
    pub fn compute_score(
        volatility: VolatilityState,
        liquidity: LiquidityState,
        leverage: LeverageState,
        correlation: Decimal,
    ) -> Decimal {
        let mut score = dec!(100.0);

        let vol_penalty = match volatility {
            VolatilityState::Normal => dec!(0.0),
            VolatilityState::Elevated => dec!(5.0),
            VolatilityState::High => dec!(15.0),
            VolatilityState::Extreme => dec!(30.0),
            VolatilityState::Collapse => dec!(50.0),
        };
        score -= vol_penalty;

        let liq_penalty = match liquidity {
            LiquidityState::Healthy => dec!(0.0),
            LiquidityState::Warning => dec!(10.0),
            LiquidityState::Danger => dec!(25.0),
            LiquidityState::Critical => dec!(45.0),
            LiquidityState::Frozen => dec!(70.0),
        };
        score -= liq_penalty;

        let lev_penalty = match leverage {
            LeverageState::Stable => dec!(0.0),
            LeverageState::Elevated => dec!(10.0),
            LeverageState::Danger => dec!(30.0),
            LeverageState::Collapse => dec!(60.0),
        };
        score -= lev_penalty;

        let corr_penalty = if correlation > dec!(0.8) {
            (correlation - dec!(0.8)) * dec!(50.0)
        } else {
            dec!(0.0)
        };
        score -= corr_penalty;

        score.clamp(dec!(0.0), dec!(100.0))
    }

    pub fn evaluate_state(score: Decimal) -> SurvivalState {
        if score >= dec!(90.0) {
            SurvivalState::Excellent
        } else if score >= dec!(70.0) {
            SurvivalState::Strong
        } else if score >= dec!(50.0) {
            SurvivalState::Moderate
        } else if score >= dec!(30.0) {
            SurvivalState::Weak
        } else if score >= dec!(10.0) {
            SurvivalState::Critical
        } else {
            SurvivalState::Failed
        }
    }
}
