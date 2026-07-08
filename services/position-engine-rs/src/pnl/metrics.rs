use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionMetrics {
    pub max_runup: Decimal,
    pub max_drawdown: Decimal,
    pub expectancy: Decimal,
    /// PnL per second of holding time; 0 if holding_secs is 0
    pub holding_efficiency: Decimal,
}

pub struct PnLMetricsEngine;

impl PnLMetricsEngine {
    /// Calculates composite position metrics.
    ///
    /// * `mfe`           – max favourable excursion (absolute price)
    /// * `mae`           – max adverse excursion (absolute price)
    /// * `realized_pnl`  – total realised PnL for this position
    /// * `win_rate`      – historical win rate for the instrument/strategy (0.0 – 1.0)
    /// * `avg_win`       – average winning trade PnL
    /// * `avg_loss`      – average losing trade PnL (positive magnitude)
    /// * `holding_secs`  – seconds the position has been open
    pub fn calculate(
        mfe: Decimal,
        mae: Decimal,
        realized_pnl: Decimal,
        win_rate: f32,
        avg_win: Decimal,
        avg_loss: Decimal,
        holding_secs: u64,
    ) -> PositionMetrics {
        // Expectancy = (win_rate × avg_win) − (loss_rate × avg_loss)
        let win_rate_dec = Decimal::from_f32_retain(win_rate).unwrap_or(Decimal::ZERO);
        let loss_rate_dec = Decimal::ONE - win_rate_dec;
        let expectancy = (win_rate_dec * avg_win) - (loss_rate_dec * avg_loss);

        // Holding efficiency = realized PnL per second; zero-safe
        let holding_efficiency = if holding_secs > 0 {
            realized_pnl / Decimal::from(holding_secs)
        } else {
            Decimal::ZERO
        };

        PositionMetrics {
            max_runup: mfe,
            max_drawdown: mae,
            expectancy,
            holding_efficiency,
        }
    }
}
