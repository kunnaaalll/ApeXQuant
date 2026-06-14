use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::types::{RegimeType, TradeCount};
use super::states::RegimeState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegimeAssessment {
    pub regime: RegimeType,
    pub trade_count: TradeCount,
    pub wins: u32,
    pub losses: u32,
    pub profit_factor: Decimal,
    pub expectancy: Decimal,
    pub average_rr: Decimal,
    pub drawdown: Decimal,
    pub stability: Decimal,
    pub state: RegimeState,
}

impl RegimeAssessment {
    pub fn new(regime: RegimeType) -> Self {
        Self {
            regime,
            trade_count: 0,
            wins: 0,
            losses: 0,
            profit_factor: Decimal::ZERO,
            expectancy: Decimal::ZERO,
            average_rr: Decimal::ZERO,
            drawdown: Decimal::ZERO,
            stability: Decimal::ZERO,
            state: RegimeState::Normal,
        }
    }
}
