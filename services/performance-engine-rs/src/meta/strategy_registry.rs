use super::strategy_state::StrategyState;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyProfile {
    pub strategy_id: Uuid,
    pub name: String,
    pub trade_count: u32,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
    pub expectancy: Decimal,
    pub max_drawdown: Decimal,
    pub stability: Decimal,
    pub confidence: Decimal,
    pub health: Decimal,
    pub state: StrategyState,
}

impl StrategyProfile {
    pub fn new(strategy_id: Uuid, name: String) -> Self {
        Self {
            strategy_id,
            name,
            trade_count: 0,
            win_rate: Decimal::ZERO,
            profit_factor: Decimal::ZERO,
            expectancy: Decimal::ZERO,
            max_drawdown: Decimal::ZERO,
            stability: Decimal::ZERO,
            confidence: Decimal::ZERO,
            health: Decimal::ZERO,
            state: StrategyState::Normal,
        }
    }
}
