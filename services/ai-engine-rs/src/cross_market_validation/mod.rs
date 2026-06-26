use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketClass {
    Forex,
    Indices,
    Metals,
    Crypto,
    Futures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossMarketScore {
    pub average_expectancy: Decimal,
    pub market_class_scores: HashMap<MarketClass, Decimal>,
    pub variance_across_markets: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferabilityScore {
    pub base_market: MarketClass,
    pub transfer_degradation_ratio: Decimal,
    pub universal_applicability_index: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobustnessScore {
    pub aggregate_robustness: Decimal,
    pub worst_performing_market: MarketClass,
    pub fail_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub strategy_id: String,
    pub cross_market_score: CrossMarketScore,
    pub transferability_score: TransferabilityScore,
    pub robustness_score: RobustnessScore,
}

pub struct CrossMarketValidator {}

impl CrossMarketValidator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn validate_strategy(
        &self,
        strategy_id: &str,
        base_market: MarketClass,
        performance_by_market: &HashMap<MarketClass, Decimal>,
    ) -> ValidationResult {
        let mut sum = Decimal::ZERO;
        let mut worst_score = Decimal::MAX;
        let mut worst_market = MarketClass::Forex;
        let mut fail_count = 0;
        
        for (&market, &score) in performance_by_market.iter() {
            sum += score;
            if score < worst_score {
                worst_score = score;
                worst_market = market;
            }
            if score < Decimal::ZERO {
                fail_count += 1;
            }
        }
        
        let avg = if performance_by_market.is_empty() {
            Decimal::ZERO
        } else {
            sum / Decimal::new(performance_by_market.len() as i64, 0)
        };
        
        let base_score = performance_by_market.get(&base_market).copied().unwrap_or(Decimal::ZERO);
        
        let degradation = if base_score > Decimal::ZERO {
            (base_score - avg) / base_score
        } else {
            Decimal::ZERO
        };

        ValidationResult {
            strategy_id: strategy_id.to_string(),
            cross_market_score: CrossMarketScore {
                average_expectancy: avg,
                market_class_scores: performance_by_market.clone(),
                variance_across_markets: Decimal::ZERO, // Placeholder for actual variance calculation
            },
            transferability_score: TransferabilityScore {
                base_market,
                transfer_degradation_ratio: degradation,
                universal_applicability_index: avg, // Simplified
            },
            robustness_score: RobustnessScore {
                aggregate_robustness: avg, // Simplified
                worst_performing_market: worst_market,
                fail_count,
            },
        }
    }
}

impl Default for CrossMarketValidator {
    fn default() -> Self {
        Self::new()
    }
}
