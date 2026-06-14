use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionMetrics {
    pub max_runup: Decimal,
    pub max_drawdown: Decimal,
    pub expectancy: Decimal,
    pub holding_efficiency: f32, // e.g., PnL per hour
}

pub struct PnLMetricsEngine;

impl PnLMetricsEngine {
    pub fn calculate(
        mfe: Decimal,
        mae: Decimal,
        realized_pnl: Decimal,
        win_rate: f32,
    ) -> PositionMetrics {
        // Naive expectancy: (win_rate * avg_win) - (loss_rate * avg_loss)
        // Here we just use a placeholder based on realized PnL and probability.
        let expectancy = realized_pnl * Decimal::from_f32_retain(win_rate).unwrap_or(Decimal::ZERO);

        PositionMetrics {
            max_runup: mfe,
            max_drawdown: mae,
            expectancy,
            holding_efficiency: 1.0, // Placeholder
        }
    }
}
