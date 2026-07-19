use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::states::TimeframeState;
use super::types::{TimeframeType, TradeCount};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeframeAssessment {
    pub timeframe: TimeframeType,
    pub trade_count: TradeCount,
    pub trade_frequency: Decimal,
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub average_rr: Decimal,
    pub drawdown: Decimal,
    pub stability: Decimal,
    pub edge_score: Decimal,
    pub state: TimeframeState,
}

impl TimeframeAssessment {
    pub fn new(timeframe: TimeframeType) -> Self {
        Self {
            timeframe,
            trade_count: 0,
            trade_frequency: Decimal::ZERO,
            expectancy: Decimal::ZERO,
            profit_factor: Decimal::ZERO,
            average_rr: Decimal::ZERO,
            drawdown: Decimal::ZERO,
            stability: Decimal::ZERO,
            edge_score: Decimal::ZERO,
            state: TimeframeState::Normal,
        }
    }
}
