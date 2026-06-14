use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::types::{Symbol, TradeCount};
use super::states::SymbolState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolAssessment {
    pub symbol: Symbol,
    pub trade_count: TradeCount,
    pub win_rate: Decimal,
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub average_rr: Decimal,
    pub drawdown: Decimal,
    pub stability: Decimal,
    pub edge_score: Decimal,
    pub state: SymbolState,
}

impl SymbolAssessment {
    pub fn new(symbol: Symbol) -> Self {
        Self {
            symbol,
            trade_count: 0,
            win_rate: Decimal::ZERO,
            expectancy: Decimal::ZERO,
            profit_factor: Decimal::ZERO,
            average_rr: Decimal::ZERO,
            drawdown: Decimal::ZERO,
            stability: Decimal::ZERO,
            edge_score: Decimal::ZERO,
            state: SymbolState::Normal,
        }
    }
}
