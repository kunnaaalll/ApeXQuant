use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::types::{SessionType, TradeCount};
use super::states::SessionState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionAssessment {
    pub session: SessionType,
    pub trade_count: TradeCount,
    pub trade_frequency: Decimal, // e.g. trades per day
    pub win_rate: Decimal,
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub average_rr: Decimal,
    pub drawdown: Decimal,
    pub stability: Decimal,
    pub state: SessionState,
}

impl SessionAssessment {
    pub fn new(session: SessionType) -> Self {
        Self {
            session,
            trade_count: 0,
            trade_frequency: Decimal::ZERO,
            win_rate: Decimal::ZERO,
            expectancy: Decimal::ZERO,
            profit_factor: Decimal::ZERO,
            average_rr: Decimal::ZERO,
            drawdown: Decimal::ZERO,
            stability: Decimal::ZERO,
            state: SessionState::Normal,
        }
    }
}
