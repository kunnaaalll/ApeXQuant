use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionScore {
    pub execution_contribution: Decimal,
    pub strategy_contribution: Decimal,
    pub market_contribution: Decimal,
    pub risk_contribution: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScore {
    pub score: u8, // 0-100
}

impl ConfidenceScore {
    pub fn new(score: u8) -> Self {
        Self {
            score: score.min(100),
        }
    }
}

pub struct AttributionEngine;

impl Default for AttributionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributionEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate_trade(
        &self,
        trade_pnl: Decimal,
        expected_pnl: Decimal,
        slippage: Decimal,
        market_volatility: Decimal,
    ) -> AttributionScore {
        // Deterministic implementation of performance attribution

        // 1. Execution is typically the cost paid to the market via slippage (usually negative)
        let execution_contribution = -slippage.abs();

        // 2. Market contribution is driven by excess volatility tailwinds
        // We heuristically attribute a fraction of the PnL to market volatility when high
        let baseline_volatility = Decimal::new(15, 2); // 0.15
        let vol_excess = if market_volatility > baseline_volatility {
            market_volatility - baseline_volatility
        } else {
            Decimal::new(0, 0)
        };
        // E.g., if vol is 0.25, excess = 0.10. We attribute 10% of PnL to market condition.
        let market_contribution = trade_pnl * vol_excess;

        // 3. Risk contribution penalizes deviations from expected return
        // If trade_pnl > expected_pnl, risk is positive (upside risk captured)
        // If trade_pnl < expected_pnl, risk is negative (downside realized)
        // We take 20% of the deviation as the risk premium
        let risk_deviation = trade_pnl - expected_pnl;
        let risk_contribution = risk_deviation * Decimal::new(20, 2); // 0.20

        // 4. Strategy contribution is the residual alpha
        let strategy_contribution =
            trade_pnl - execution_contribution - market_contribution - risk_contribution;

        AttributionScore {
            execution_contribution,
            strategy_contribution,
            market_contribution,
            risk_contribution,
        }
    }
}
