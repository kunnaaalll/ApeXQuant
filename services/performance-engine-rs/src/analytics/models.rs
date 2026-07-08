use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Comprehensive analytics result combining all computed metrics for a strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAnalyticsResult {
    pub strategy_id: String,
    pub trade_count: u32,
    pub win_count: u32,
    pub loss_count: u32,
    pub breakeven_count: u32,
    // Returns
    pub net_profit: Decimal,
    pub gross_profit: Decimal,
    pub gross_loss: Decimal,
    // Rate metrics
    pub win_rate: Decimal,
    pub loss_rate: Decimal,
    pub profit_factor: Decimal,
    // Per-trade metrics
    pub expectancy: Decimal,
    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
    pub average_rr: Decimal,
    pub sqn: Decimal,
    // Streak metrics
    pub max_consecutive_wins: u32,
    pub max_consecutive_losses: u32,
    // Duration metrics
    pub average_duration_seconds: i64,
    // Drawdown
    pub max_drawdown: Decimal,
    pub average_drawdown: Decimal,
    pub recovery_factor: Decimal,
    pub ulcer_index: Decimal,
    // Risk-adjusted
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub calmar_ratio: Decimal,
    pub omega_ratio: Decimal,
    pub burke_ratio: Decimal,
    pub sterling_ratio: Decimal,
    pub mar_ratio: Decimal,
    // Edge & health
    pub edge_score: Decimal,
    pub confidence: Decimal,
    pub stability: Decimal,
    pub health_score: u8,
}

impl Default for StrategyAnalyticsResult {
    fn default() -> Self {
        Self {
            strategy_id: String::new(),
            trade_count: 0,
            win_count: 0,
            loss_count: 0,
            breakeven_count: 0,
            net_profit: Decimal::ZERO,
            gross_profit: Decimal::ZERO,
            gross_loss: Decimal::ZERO,
            win_rate: Decimal::ZERO,
            loss_rate: Decimal::ZERO,
            profit_factor: Decimal::ZERO,
            expectancy: Decimal::ZERO,
            average_win: Decimal::ZERO,
            average_loss: Decimal::ZERO,
            largest_win: Decimal::ZERO,
            largest_loss: Decimal::ZERO,
            average_rr: Decimal::ZERO,
            sqn: Decimal::ZERO,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
            average_duration_seconds: 0,
            max_drawdown: Decimal::ZERO,
            average_drawdown: Decimal::ZERO,
            recovery_factor: Decimal::ZERO,
            ulcer_index: Decimal::ZERO,
            sharpe_ratio: Decimal::ZERO,
            sortino_ratio: Decimal::ZERO,
            calmar_ratio: Decimal::ZERO,
            omega_ratio: Decimal::ZERO,
            burke_ratio: Decimal::ZERO,
            sterling_ratio: Decimal::ZERO,
            mar_ratio: Decimal::ZERO,
            edge_score: Decimal::ZERO,
            confidence: Decimal::ZERO,
            stability: Decimal::ZERO,
            health_score: 0,
        }
    }
}
