use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAttribution {
    pub allocation_effect: Decimal,
    pub selection_effect: Decimal,
    pub interaction_effect: Decimal,
    pub total_excess_return: Decimal,
}

pub struct AttributionEngine;

impl Default for AttributionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributionEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_brinson_fachler(
        &self,
        portfolio_weight: Decimal,
        benchmark_weight: Decimal,
        portfolio_return: Decimal,
        benchmark_return: Decimal,
        total_benchmark_return: Decimal,
    ) -> PerformanceAttribution {
        let allocation_effect =
            (portfolio_weight - benchmark_weight) * (benchmark_return - total_benchmark_return);
        let selection_effect = benchmark_weight * (portfolio_return - benchmark_return);
        let interaction_effect =
            (portfolio_weight - benchmark_weight) * (portfolio_return - benchmark_return);
        let total_excess_return = allocation_effect + selection_effect + interaction_effect;

        PerformanceAttribution {
            allocation_effect,
            selection_effect,
            interaction_effect,
            total_excess_return,
        }
    }
}
