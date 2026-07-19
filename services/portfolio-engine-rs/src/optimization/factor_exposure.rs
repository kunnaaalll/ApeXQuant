use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskFactor {
    Market,
    Size,
    Value,
    Momentum,
    Quality,
    Volatility,
    Yield,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FactorExposures {
    pub exposures: HashMap<RiskFactor, Decimal>,
}

pub struct FactorEngine;

impl Default for FactorEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FactorEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_portfolio_factors(
        &self,
        asset_weights: &[(String, Decimal)],
        asset_factor_loadings: &HashMap<String, HashMap<RiskFactor, Decimal>>,
    ) -> FactorExposures {
        let mut portfolio_exposures = HashMap::new();

        for (symbol, weight) in asset_weights {
            if let Some(loadings) = asset_factor_loadings.get(symbol) {
                for (factor, loading) in loadings {
                    let current = portfolio_exposures
                        .entry(factor.clone())
                        .or_insert(Decimal::ZERO);
                    *current += weight * loading;
                }
            }
        }

        FactorExposures {
            exposures: portfolio_exposures,
        }
    }
}
