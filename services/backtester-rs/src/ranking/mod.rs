//! Ranking Module
//!
//! Evaluates and ranks strategies across different scopes such as globally,
//! per symbol, per session, or per timeframe.

use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct GlobalStrategyRank {
    pub strategy_id: String,
    pub rank: usize,
    pub score: Decimal,
}

#[derive(Debug, Clone)]
pub struct SymbolRank {
    pub symbol: String,
    pub strategy_id: String,
    pub rank: usize,
    pub score: Decimal,
}

#[derive(Debug, Clone)]
pub struct SessionRank {
    pub session_name: String,
    pub strategy_id: String,
    pub rank: usize,
    pub score: Decimal,
}

#[derive(Debug, Clone)]
pub struct TimeframeRank {
    pub timeframe: String,
    pub strategy_id: String,
    pub rank: usize,
    pub score: Decimal,
}

pub struct RankingEngine;

impl RankingEngine {
    pub fn rank_global(_strategies: &[String]) -> Result<Vec<GlobalStrategyRank>, &'static str> {
        // Stub: evaluate and rank all strategies globally
        Ok(vec![])
    }
}
