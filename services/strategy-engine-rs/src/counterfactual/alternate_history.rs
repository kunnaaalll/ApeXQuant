use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualResult {
    pub scenario_name: String,
    pub original_profit: Decimal,
    pub simulated_profit: Decimal,
}

pub struct AlternateHistoryEngine;

impl AlternateHistoryEngine {
    /// Calculate simulated vs original returns under high-slippage conditions
    pub fn simulate_high_slippage(
        original_profit: Decimal,
        trades_count: u32,
        slippage_per_trade: Decimal,
    ) -> CounterfactualResult {
        let total_slippage = Decimal::from(trades_count) * slippage_per_trade;
        let simulated_profit = original_profit - total_slippage;

        CounterfactualResult {
            scenario_name: "high_slippage".to_string(),
            original_profit,
            simulated_profit,
        }
    }
}
