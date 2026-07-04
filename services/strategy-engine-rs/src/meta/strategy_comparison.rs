use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyComparisonReport {
    pub strategy_a: String,
    pub strategy_b: String,
    pub expectancy_diff: Decimal,
    pub profit_factor_diff: Decimal,
}

pub struct StrategyComparer;

impl StrategyComparer {
    pub fn compare(
        id_a: &str,
        expectancy_a: Decimal,
        pf_a: Decimal,
        id_b: &str,
        expectancy_b: Decimal,
        pf_b: Decimal,
    ) -> StrategyComparisonReport {
        StrategyComparisonReport {
            strategy_a: id_a.to_string(),
            strategy_b: id_b.to_string(),
            expectancy_diff: expectancy_a - expectancy_b,
            profit_factor_diff: pf_a - pf_b,
        }
    }
}
